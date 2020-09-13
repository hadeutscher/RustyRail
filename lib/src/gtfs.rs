/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod opener;

use crate::HaError;
use crate::JSON;
use chrono::{Datelike, Duration, NaiveDate};
use json::JsonValue;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::result::Result;
use zip::ZipArchive;

/// A unique identifier type for trains in the database
pub type TrainId = String;

/// A unique identifier type for stations in the database
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
            return Err(Box::new(HaError::GTFSError(format!("{} header not found", stringify!($x)))));
        }
        )+
        ($( $x.unwrap(), )+)
    }}
}

/// Represents a database train station entry
#[derive(Serialize, Deserialize)]
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
        write!(f, "{}: {}", self.id, self.name)
    }
}

impl JSON for Station {
    fn to_json(&self) -> JsonValue {
        object! {
            id: self.id,
            name: self.name.to_owned()
        }
    }
}

impl Station {
    /// Create a new Station object
    pub fn new(id: StationId, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }

    /// Gets the station identifier
    pub fn id(&self) -> StationId {
        self.id
    }

    /// Gets the station name
    pub fn name(&self) -> &String {
        &self.name
    }
}

/// Represents a duration in seconds. Used instead of chrono::Duration since the latter doesn't support serde.
#[derive(Copy, Clone)]
pub struct HaDuration {
    seconds: u64,
}

impl HaDuration {
    /// Create a new HaDuration object from hours, minutes and seconds
    pub fn from_hms(h: u32, m: u32, s: u32) -> Self {
        HaDuration {
            seconds: (h as u64) * 3600 + (m as u64) * 60 + s as u64,
        }
    }

    /// Create a new Haduration object from seconds only
    pub fn from_seconds(s: u64) -> Self {
        HaDuration { seconds: s }
    }

    /// Convert to a chrono duration
    ///
    /// Examples:
    /// ```
    /// use harail::HaDuration;
    /// use chrono::Duration;
    ///
    /// let d = HaDuration::from_hms(10, 30, 40);
    /// let c = Duration::hours(10) + Duration::minutes(30) + Duration::seconds(40);
    /// assert_eq!(c, d.to_chrono());
    /// ```
    pub fn to_chrono(&self) -> Duration {
        Duration::seconds(self.seconds as i64)
    }
}

impl fmt::Display for HaDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.seconds / 3600,
            (self.seconds % 3600) / 60,
            self.seconds % 60
        )
    }
}

impl Serialize for HaDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.seconds)
    }
}

struct HaDurationVisitor;

impl<'de> Visitor<'de> for HaDurationVisitor {
    type Value = HaDuration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 2^32")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(HaDuration::from_seconds(value))
    }
}

impl<'de> Deserialize<'de> for HaDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(HaDurationVisitor)
    }
}

/// Represents a train's scheduled stopping at a certain station
#[derive(Serialize, Deserialize)]
pub struct StopSchedule {
    station: StationId,
    arrival_offset: HaDuration,
    departure_offset: HaDuration,
}

impl StopSchedule {
    /// Create a new Stop object
    pub fn new(
        station: StationId,
        arrival_offset: HaDuration,
        departure_offset: Option<HaDuration>,
    ) -> Self {
        Self {
            station,
            arrival_offset,
            departure_offset: departure_offset.unwrap_or(arrival_offset),
        }
    }

    /// The station at which the train stopped
    pub fn station(&self) -> StationId {
        self.station
    }

    /// The time the train has arrived at the station, as offset from the start of the schedule
    pub fn arrival_offset(&self) -> HaDuration {
        self.arrival_offset
    }

    /// The time the train has departed from the station, as offset from the start of the schedule.
    ///
    /// This is usually the same as arrival offset, unless the train waits at the station.
    pub fn departure_offset(&self) -> HaDuration {
        self.departure_offset
    }
}

struct PrototypeTrain {
    id: TrainId,
    stops: Vec<Option<StopSchedule>>,
    dates: Vec<NaiveDate>,
}

/// Represents a single train's schedule
///
/// Note that this objects represents not the train but rather the act of the train moving from its initial station to its end station, possibly passing through other stations, repeatedly over a number of days.
/// For example, one physical train might be responsible for handling a line repetitively, traveling forward and backwards over it many times a day.
/// Each such pass over this route from start to end (or vice versa) is represented by a Train object.
#[derive(Serialize, Deserialize)]
pub struct Train {
    id: TrainId,
    stops: Vec<StopSchedule>,
    dates: Vec<NaiveDate>,
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
    /// Create a new Train object
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            stops: Vec::new(),
            dates: Vec::new(),
        }
    }

    /// Create a train object with certain stops on a single day
    pub fn from_stops_date(id: &str, stops: Vec<StopSchedule>, date: NaiveDate) -> Self {
        Self {
            id: id.to_owned(),
            stops,
            dates: vec![date],
        }
    }

    /// Create a train object with certain stops on multiple days
    pub fn from_stops_dates(id: &str, stops: Vec<StopSchedule>, dates: Vec<NaiveDate>) -> Self {
        Self {
            id: id.to_owned(),
            stops,
            dates,
        }
    }

    /// Get the train identifier
    pub fn id(&self) -> &TrainId {
        &self.id
    }

    /// Iterate over the train stops
    pub fn stops(&self) -> impl Iterator<Item = &StopSchedule> {
        self.stops.iter()
    }

    /// Iterate over the train schedule days
    pub fn dates(&self) -> impl Iterator<Item = &NaiveDate> {
        self.dates.iter()
    }
}

/// A database of all available trains and stations
#[derive(Serialize, Deserialize)]
pub struct RailroadData {
    stations: HashMap<StationId, Station>,
    trains: HashMap<TrainId, Train>,
}

impl RailroadData {
    /// Create a new RailroadData object
    pub fn new() -> Self {
        RailroadData {
            stations: HashMap::new(),
            trains: HashMap::new(),
        }
    }

    /// Create a new RailroadData object with some stations and trains
    pub fn from_stations_trains(stations: Vec<Station>, trains: Vec<Train>) -> Self {
        let mut result = Self::new();
        stations.into_iter().for_each(|x| {
            result.stations.insert(x.id, x);
        });
        trains.into_iter().for_each(|x| {
            result.trains.insert(x.id.to_owned(), x);
        });
        result
    }

    /// Get the station with the given identifier
    pub fn station(&self, id: StationId) -> Option<&Station> {
        self.stations.get(&id)
    }

    /// Get the train with the given identifier
    pub fn train(&self, id: &str) -> Option<&Train> {
        self.trains.get(id)
    }

    /// Iterates over the stations in the database
    pub fn stations(&self) -> impl Iterator<Item = &Station> {
        self.stations.values()
    }

    /// Iterates over the trains in the database
    pub fn trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.values()
    }

    /// Finds a station with the given name.
    ///
    /// Examples:
    /// ```
    /// use harail::{RailroadData, Station};
    ///
    /// let data = RailroadData::from_stations_trains(vec![Station::new(100, "test")], vec![]);
    /// let station = data.find_station("test").unwrap();
    /// assert_eq!(100, station.id());
    /// ```
    pub fn find_station(&self, name: &str) -> Option<&Station> {
        for station in self.stations.values() {
            if station.name == name {
                return Some(station);
            }
        }
        None
    }

    /// Gets the start date of the database
    pub fn start_date(&self) -> Option<NaiveDate> {
        let mut result: Option<NaiveDate> = None;
        for train in self.trains.values() {
            for date in &train.dates {
                if let Some(curr) = result {
                    if date < &curr {
                        result = Some(*date);
                    }
                } else {
                    result = Some(*date);
                }
            }
        }
        result
    }

    /// Gets the end date of the database
    pub fn end_date(&self) -> Option<NaiveDate> {
        let mut result: Option<NaiveDate> = None;
        for train in self.trains.values() {
            for date in &train.dates {
                if let Some(curr) = result {
                    if date > &curr {
                        result = Some(*date);
                    }
                } else {
                    result = Some(*date);
                }
            }
        }
        result
    }

    fn parse_agency<R: Read>(reader: R) -> Result<u64, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let (agency_id, agency_name) = headers!(reader.headers()?, agency_id, agency_name);
        for result in reader.records() {
            let record = result?;
            let agency_name = record.get(agency_name).ok_or_else(|| "agency_name")?;
            if agency_name == "רכבת ישראל" {
                let agency_id: u64 = record
                    .get(agency_id)
                    .ok_or_else(|| HaError::GTFSError("agency_id".to_owned()))?
                    .parse()?;
                return Ok(agency_id);
            }
        }
        Err(Box::new(HaError::GTFSError("not found".to_owned())))
    }

    fn parse_routes<R: Read>(reader: R, irw_id: u64) -> Result<HashSet<u64>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let (route_id, agency_id) = headers!(reader.headers()?, route_id, agency_id);
        let mut set = HashSet::new();
        let irw_id_str = irw_id.to_string();
        for result in reader.records() {
            let record = result?;
            let agency_id = record
                .get(agency_id)
                .ok_or_else(|| HaError::GTFSError("agency_id".to_owned()))?;
            if agency_id == irw_id_str {
                let route_id: u64 = record
                    .get(route_id)
                    .ok_or_else(|| HaError::GTFSError("route_id".to_owned()))?
                    .parse()?;
                set.insert(route_id);
            }
        }
        Ok(set)
    }

    fn parse_stops<R: Read>(
        &mut self,
        reader: R,
        irw_stops: HashSet<StationId>,
    ) -> Result<(), Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let (stop_id, stop_name) = headers!(reader.headers()?, stop_id, stop_name);
        for result in reader.records() {
            let record = result?;
            let stop_id: u64 = record
                .get(stop_id)
                .ok_or_else(|| HaError::GTFSError("stop_id".to_owned()))?
                .parse()?;
            if !irw_stops.contains(&stop_id) {
                continue;
            }
            let stop_name = record
                .get(stop_name)
                .ok_or_else(|| HaError::GTFSError("stop_name".to_owned()))?;
            self.stations
                .insert(stop_id, Station::new(stop_id, stop_name));
        }
        Ok(())
    }

    fn parse_gtfs_date(date: &str) -> Result<NaiveDate, Box<dyn Error>> {
        let date_num: u32 = date.parse()?;
        Ok(NaiveDate::from_ymd(
            (date_num / 10000) as i32,
            (date_num % 10000) / 100,
            date_num % 100,
        ))
    }

    fn parse_gtfs_daymap(period: (NaiveDate, NaiveDate), daymap: [bool; 7]) -> Vec<NaiveDate> {
        let mut result = Vec::new();
        let mut date = period.0;
        while date <= period.1 {
            if daymap[date.weekday().num_days_from_sunday() as usize] {
                result.push(date);
            }
            date = date.succ();
        }
        result
    }

    fn parse_calendar<R: Read>(reader: R) -> Result<HashMap<u64, Vec<NaiveDate>>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
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
            reader.headers()?,
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
        let mut map = HashMap::new();
        for result in reader.records() {
            let record = result?;
            let service_id: u64 = record
                .get(service_id)
                .ok_or_else(|| HaError::GTFSError("service_id".to_owned()))?
                .parse()?;
            let start_date = Self::parse_gtfs_date(
                record
                    .get(start_date)
                    .ok_or_else(|| HaError::GTFSError("start_date".to_owned()))?,
            )?;
            let end_date = Self::parse_gtfs_date(
                record
                    .get(end_date)
                    .ok_or_else(|| HaError::GTFSError("end_date".to_owned()))?,
            )?;
            let daymap = [
                record.get(sunday).unwrap_or("0") == "1",
                record.get(monday).unwrap_or("0") == "1",
                record.get(tuesday).unwrap_or("0") == "1",
                record.get(wednesday).unwrap_or("0") == "1",
                record.get(thursday).unwrap_or("0") == "1",
                record.get(friday).unwrap_or("0") == "1",
                record.get(saturday).unwrap_or("0") == "1",
            ];
            map.insert(
                service_id,
                Self::parse_gtfs_daymap((start_date, end_date), daymap),
            );
        }
        Ok(map)
    }

    fn parse_trips<R: Read>(
        reader: R,
        irw_routes: HashSet<u64>,
        services: HashMap<u64, Vec<NaiveDate>>,
    ) -> Result<HashMap<String, Option<Vec<NaiveDate>>>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let (route_id, trip_id, service_id) =
            headers!(reader.headers()?, route_id, trip_id, service_id);
        let mut map = HashMap::new();
        for result in reader.records() {
            let record = result?;
            let route_id: u64 = record
                .get(route_id)
                .ok_or_else(|| HaError::GTFSError("route_id".to_owned()))?
                .parse()?;
            if !irw_routes.contains(&route_id) {
                continue;
            }
            let service_id: u64 = record
                .get(service_id)
                .ok_or_else(|| HaError::GTFSError("service_id".to_owned()))?
                .parse()?;
            if let Some(dates) = services.get(&service_id) {
                let trip_id = record
                    .get(trip_id)
                    .ok_or_else(|| HaError::GTFSError("trip_id".to_owned()))?;
                map.insert(trip_id.to_owned(), Some(dates.clone()));
            }
        }
        Ok(map)
    }

    fn parse_gtfs_time(time_str: &str) -> Result<HaDuration, Box<dyn Error>> {
        let mut state = 0;
        let (mut h, mut m, mut s): (u32, u32, u32) = (0, 0, 0);
        for part in time_str.split(":") {
            match state {
                0 => h = part.parse()?,
                1 => m = part.parse()?,
                2 => s = part.parse()?,
                _ => {
                    return Err(Box::new(HaError::GTFSError(
                        "Invalid date format".to_owned(),
                    )))
                }
            };
            state += 1;
        }
        Ok(HaDuration::from_hms(h, m, s))
    }

    fn parse_stop_times<R: Read>(
        &mut self,
        reader: R,
        mut trips: HashMap<String, Option<Vec<NaiveDate>>>,
    ) -> Result<HashSet<u64>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let (trip_id, arrival_time, departure_time, stop_id, stop_sequence) = headers!(
            reader.headers()?,
            trip_id,
            arrival_time,
            departure_time,
            stop_id,
            stop_sequence
        );
        let mut stations = HashSet::new();
        let mut proto_trains = HashMap::new();
        for result in reader.records() {
            let record = result?;
            let trip_id = record
                .get(trip_id)
                .ok_or_else(|| HaError::GTFSError("trip_id".to_owned()))?;
            if !trips.contains_key(trip_id) {
                continue;
            }
            let arrival_time = record
                .get(arrival_time)
                .ok_or_else(|| HaError::GTFSError("arrival_time".to_owned()))?;
            let arrival_datetime = Self::parse_gtfs_time(arrival_time)?;
            let departure_time = record
                .get(departure_time)
                .ok_or_else(|| HaError::GTFSError("departure_time".to_owned()))?;
            let departure_datetime = Self::parse_gtfs_time(departure_time)?;
            let stop_id: u64 = record
                .get(stop_id)
                .ok_or_else(|| HaError::GTFSError("stop_id".to_owned()))?
                .parse()?;
            let stop_sequence: u64 = record
                .get(stop_sequence)
                .ok_or_else(|| HaError::GTFSError("stop_sequence".to_owned()))?
                .parse()?;
            if stop_sequence == 0 {
                return Err(Box::new(HaError::GTFSError(
                    "stop_sequence == 0".to_owned(),
                )));
            }
            let stop_seq_index = stop_sequence as usize - 1;
            let stop = StopSchedule::new(stop_id, arrival_datetime, Some(departure_datetime));
            if !proto_trains.contains_key(trip_id) {
                // We take ownership of the dates vector from inside the trips table by replacing it with None.
                // This should never panic because insert will never return None since we validated trips.contains_key(trip_id) before,
                // and the optional vec is always set to Some by parse_trips, and only replaced once by us (we validate !proto_trains.contains_key(trip_id) here)
                let dates = trips.insert(trip_id.to_owned(), None).unwrap().unwrap();
                proto_trains.insert(
                    trip_id.to_owned(),
                    PrototypeTrain {
                        id: trip_id.to_owned(),
                        stops: Vec::new(),
                        dates,
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
                return Err(Box::new(HaError::GTFSError(format!(
                    "partial train: {}",
                    id
                ))));
            }
            let train = Train {
                id: ptrain.id,
                stops: ptrain.stops.into_iter().map(|x| x.unwrap()).collect(),
                dates: ptrain.dates,
            };
            self.trains.insert(id, train);
        }
        Ok(stations)
    }

    fn load_gtfs<T: for<'a> opener::FileOpener<'a>>(mut opener: T) -> Result<Self, Box<dyn Error>> {
        let irw_id = Self::parse_agency(opener.open("agency.txt")?)?;
        let irw_routes = Self::parse_routes(opener.open("routes.txt")?, irw_id)?;
        let services = Self::parse_calendar(opener.open("calendar.txt")?)?;
        let irw_trips = Self::parse_trips(opener.open("trips.txt")?, irw_routes, services)?;
        let mut result = Self::new();
        let irw_stops = result.parse_stop_times(opener.open("stop_times.txt")?, irw_trips)?;
        result.parse_stops(opener.open("stops.txt")?, irw_stops)?;
        Ok(result)
    }

    /// Loads a GTFS file database from a directory containing GTFS text files.
    pub fn from_gtfs_directory(root: &Path) -> Result<Self, Box<dyn Error>> {
        let opener = opener::PathFileOpener::new(root);
        Self::load_gtfs(opener)
    }

    /// Loads a GTFS file database from a zip file containing GTFS text files.
    pub fn from_gtfs_zip(root: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(root)?;
        let reader = BufReader::new(file);
        let zip = ZipArchive::new(reader)?;
        let opener = opener::ZipFileOpener::new(zip);
        Self::load_gtfs(opener)
    }
}
