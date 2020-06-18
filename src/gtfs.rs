/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::result::Result;

use super::errors::make_error;

pub type TrainId = String;
pub type StationId = u64;

macro_rules! headers {
    ($h:expr, $( $x:ident ), +) => {{
        $(
        let mut $x : Option<usize> = None;
        )+
        for (i, h) in $h.iter().enumerate() {
            match h {
                $(
                stringify!($x) => $x = Some(i),
                )+
                &_ => {}
            }
        }
        $(
        if $x.is_none() {
            return Err(make_error(&format!("{} header not found", stringify!($x))));
        }
        )+
        ($( $x.unwrap(), )+)
    }}
}

pub struct Station {
    id: StationId,
    name: String,
}

impl PartialEq for Station {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Station {}

impl Hash for Station {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.id(), self.name())
    }
}

impl Station {
    pub fn new(id: StationId, name: String) -> Self {
        Self { id: id, name: name }
    }

    pub fn id(&self) -> StationId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Stop {
    station: StationId,
    arrival: NaiveDateTime,
    departure: NaiveDateTime,
}

impl Stop {
    pub fn new(station: StationId, arrival: NaiveDateTime, departure: NaiveDateTime) -> Self {
        Self {
            station,
            arrival,
            departure,
        }
    }

    pub fn station(&self) -> StationId {
        self.station
    }

    pub fn arrival(&self) -> NaiveDateTime {
        self.arrival
    }

    pub fn departure(&self) -> NaiveDateTime {
        self.departure
    }
}

struct PrototypeTrain {
    id: TrainId,
    stops: Vec<Option<Stop>>,
}

pub struct Train {
    id: TrainId,
    stops: Vec<Stop>,
}

impl PartialEq for Train {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Train {}

impl Hash for Train {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Train {
    pub fn new(id: TrainId) -> Self {
        Self {
            id: id,
            stops: Vec::new(),
        }
    }

    pub fn id(&self) -> &TrainId {
        &self.id
    }

    pub fn stops(&self) -> impl Iterator<Item = &Stop> {
        self.stops.iter()
    }
}

pub struct RailroadData {
    stations: HashMap<StationId, Station>,
    trains: HashMap<TrainId, Train>,
}

impl RailroadData {
    pub fn station(&self, id: StationId) -> &Station {
        &self.stations[&id]
    }

    pub fn train(&self, id: &TrainId) -> &Train {
        &self.trains[id]
    }

    pub fn stations(&self) -> impl Iterator<Item = &Station> {
        self.stations.values()
    }

    pub fn trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.values()
    }

    pub fn find_station(&self, name: &str) -> Option<&Station> {
        for station in self.stations.values() {
            if station.name == name {
                return Some(station);
            }
        }
        None
    }

    fn parse_agency(root: &Path) -> Result<u64, Box<dyn Error>> {
        let file = File::open(root.join("agency.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (agency_id, agency_name) = headers!(rdr.headers()?, agency_id, agency_name);
        for result in rdr.records() {
            let record = result?;
            let agency_name = record.get(agency_name).ok_or_else(|| "agency_name")?;
            if agency_name == "רכבת ישראל" {
                let agency_id: u64 = record
                    .get(agency_id)
                    .ok_or_else(|| make_error("agency_id"))?
                    .parse()?;
                return Ok(agency_id);
            }
        }
        Err(make_error("not found"))
    }

    fn parse_routes(root: &Path, irw_id: u64) -> Result<HashSet<u64>, Box<dyn Error>> {
        let file = File::open(root.join("routes.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (route_id, agency_id) = headers!(rdr.headers()?, route_id, agency_id);
        let mut set = HashSet::new();
        let irw_id_str = irw_id.to_string();
        for result in rdr.records() {
            let record = result?;
            let agency_id = record
                .get(agency_id)
                .ok_or_else(|| make_error("agency_id"))?;
            if agency_id == irw_id_str {
                let route_id: u64 = record
                    .get(route_id)
                    .ok_or_else(|| make_error("route_id"))?
                    .parse()?;
                set.insert(route_id);
            }
        }
        Ok(set)
    }

    fn parse_stops(
        &mut self,
        root: &Path,
        irw_stops: &HashSet<StationId>,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::open(root.join("stops.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (stop_id, stop_name) = headers!(rdr.headers()?, stop_id, stop_name);
        for result in rdr.records() {
            let record = result?;
            let stop_id: u64 = record
                .get(stop_id)
                .ok_or_else(|| make_error("stop_id"))?
                .parse()?;
            if !irw_stops.contains(&stop_id) {
                continue;
            }
            let stop_name = record
                .get(stop_name)
                .ok_or_else(|| make_error("stop_name"))?;
            self.stations
                .insert(stop_id, Station::new(stop_id, stop_name.to_owned()));
        }
        Ok(())
    }

    fn parse_calendar(root: &Path, date: NaiveDate) -> Result<HashSet<u64>, Box<dyn Error>> {
        let file = File::open(root.join("calendar.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (
            service_id,
            sunday,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            start_date,
            end_date,
        ) = headers!(
            rdr.headers()?,
            service_id,
            sunday,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            start_date,
            end_date
        );
        let day_header = match date.weekday() {
            chrono::Weekday::Sun => sunday,
            chrono::Weekday::Mon => monday,
            chrono::Weekday::Tue => tuesday,
            chrono::Weekday::Wed => wednesday,
            chrono::Weekday::Thu => thursday,
            chrono::Weekday::Fri => friday,
            chrono::Weekday::Sat => saturday,
        };
        let date_num =
            (date.year() as u64 * 10000) + (date.month() as u64) * 100 + date.day() as u64;
        let mut set = HashSet::new();
        for result in rdr.records() {
            let record = result?;
            let service_id: u64 = record
                .get(service_id)
                .ok_or_else(|| make_error("service_id"))?
                .parse()?;
            let start_date: u64 = record
                .get(start_date)
                .ok_or_else(|| make_error("start_date"))?
                .parse()?;
            let end_date: u64 = record
                .get(end_date)
                .ok_or_else(|| make_error("end_date"))?
                .parse()?;
            let day_availability: u64 = record
                .get(day_header)
                .ok_or_else(|| make_error("days"))?
                .parse()?;
            /* Note that end date is inclusive */
            if day_availability > 0 && start_date <= date_num && date_num <= end_date {
                set.insert(service_id);
            }
        }
        Ok(set)
    }

    fn parse_trips(
        root: &Path,
        irw_routes: &HashSet<u64>,
        services: &HashSet<u64>,
    ) -> Result<HashSet<String>, Box<dyn Error>> {
        let file = File::open(root.join("trips.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (route_id, trip_id, service_id) =
            headers!(rdr.headers()?, route_id, trip_id, service_id);
        let mut set = HashSet::new();
        for result in rdr.records() {
            let record = result?;
            let route_id: u64 = record
                .get(route_id)
                .ok_or_else(|| make_error("route_id"))?
                .parse()?;
            if !irw_routes.contains(&route_id) {
                continue;
            }
            let service_id: u64 = record
                .get(service_id)
                .ok_or_else(|| make_error("service_id"))?
                .parse()?;
            if !services.contains(&service_id) {
                continue;
            }
            let trip_id = record
                .get(trip_id)
                .ok_or_else(|| make_error("service_id"))?;
            set.insert(trip_id.to_owned());
        }
        Ok(set)
    }

    fn parse_irw_time(
        mut date: NaiveDate,
        time_str: &str,
    ) -> Result<NaiveDateTime, Box<dyn Error>> {
        let mut state = 0;
        let (mut h, mut m, mut s): (u32, u32, u32) = (0, 0, 0);
        for part in time_str.split(":") {
            match state {
                0 => h = part.parse()?,
                1 => m = part.parse()?,
                2 => s = part.parse()?,
                _ => return Err(make_error("Invalid date format")),
            };
            state += 1;
        }
        if h >= 24 {
            date += chrono::Duration::days((h / 24) as i64);
            h = h % 24;
        }
        Ok(NaiveDateTime::new(date, NaiveTime::from_hms(h, m, s)))
    }

    fn parse_stop_times(
        &mut self,
        root: &Path,
        trips: &HashSet<String>,
        date: NaiveDate,
        stations: &mut HashSet<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::open(root.join("stop_times.txt"))?;
        let mut rdr = csv::Reader::from_reader(file);
        let (trip_id, arrival_time, departure_time, stop_id, stop_sequence) = headers!(
            rdr.headers()?,
            trip_id,
            arrival_time,
            departure_time,
            stop_id,
            stop_sequence
        );
        let mut proto_trains = HashMap::new();
        for result in rdr.records() {
            let record = result?;
            let trip_id = record.get(trip_id).ok_or_else(|| make_error("trip_id"))?;
            if !trips.contains(trip_id) {
                continue;
            }
            let arrival_time = record
                .get(arrival_time)
                .ok_or_else(|| make_error("arrival_time"))?;
            let arrival_datetime = Self::parse_irw_time(date, arrival_time)?;
            let departure_time = record
                .get(departure_time)
                .ok_or_else(|| make_error("departure_time"))?;
            let departure_datetime = Self::parse_irw_time(date, departure_time)?;
            let stop_id: u64 = record
                .get(stop_id)
                .ok_or_else(|| make_error("stop_id"))?
                .parse()?;
            let stop_sequence: u64 = record
                .get(stop_sequence)
                .ok_or_else(|| make_error("stop_sequence"))?
                .parse()?;
            if stop_sequence == 0 {
                return Err(make_error("stop_sequence == 0"));
            }
            let stop_seq_index = stop_sequence as usize - 1;
            let stop = Stop::new(stop_id, arrival_datetime, departure_datetime);
            if !proto_trains.contains_key(trip_id) {
                proto_trains.insert(
                    trip_id.to_owned(),
                    PrototypeTrain {
                        id: trip_id.to_owned(),
                        stops: Vec::new(),
                    },
                );
            }
            let train = proto_trains.get_mut(trip_id).unwrap();
            if train.stops.len() > stop_seq_index {
                train.stops[stop_seq_index] = Some(stop);
            } else if train.stops.len() < stop_seq_index {
                train.stops.resize_with(stop_seq_index + 1, || None);
                train.stops[stop_seq_index] = Some(stop);
            } else {
                train.stops.push(Some(stop));
            }
            stations.insert(stop_id);
        }
        for (id, ptrain) in proto_trains {
            if ptrain.stops.iter().any(|x| x.is_none()) {
                return Err(make_error(&format!("partial train: {}", id)));
            }
            let train = Train {
                id: ptrain.id,
                stops: ptrain.stops.into_iter().map(|x| x.unwrap()).collect(),
            };
            self.trains.insert(id, train);
        }
        Ok(())
    }

    pub fn from_gtfs(
        root: &Path,
        period: (NaiveDateTime, NaiveDateTime),
    ) -> Result<Self, Box<dyn Error>> {
        let irw_id = Self::parse_agency(root)?;
        let irw_routes = Self::parse_routes(root, irw_id)?;
        let mut result = Self {
            stations: HashMap::new(),
            trains: HashMap::new(),
        };
        let mut date = period.0.date();
        let end_date = period.1.date();
        let mut stations = HashSet::new();
        while date < end_date {
            let services = Self::parse_calendar(root, date)?;
            let trips = Self::parse_trips(root, &irw_routes, &services)?;
            result.parse_stop_times(root, &trips, date, &mut stations)?;
            date = date.succ();
        }
        result.parse_stops(root, &stations)?;
        Ok(result)
    }
}