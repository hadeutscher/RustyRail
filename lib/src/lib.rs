/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod errors;
mod graph;
mod gtfs;

#[macro_use(object)]
extern crate json;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use json::JsonValue;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub use errors::HaError;
pub use gtfs::{HaDuration, RailroadData, Station, StationId, StopSchedule, Train, TrainId};

/// An object which can be written to JSON.
///
/// This is different than the Serialize trait, since it doesn't guarantee that the writing function is one to one. It is more like the Display trait, just with JSON instead of human readable strings.
pub trait JSON {
    fn to_json(&self) -> JsonValue;
}

/// Represents a train stopping at a certain station
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Stop<'a> {
    station: &'a Station,
    arrival: NaiveDateTime,
    departure: NaiveDateTime,
}

impl<'a> JSON for Stop<'a> {
    fn to_json(&self) -> JsonValue {
        let arrival = DateTime::<Utc>::from_utc(self.arrival(), Utc);
        let departure = DateTime::<Utc>::from_utc(self.departure(), Utc);
        object! {
            station: self.station.id().to_owned(),
            arrival: arrival.to_rfc3339(),
            departure: departure.to_rfc3339(),
        }
    }
}

impl<'a> Stop<'a> {
    fn inflate_stop_time(date: NaiveDate, offset: HaDuration) -> NaiveDateTime {
        NaiveDateTime::new(date, NaiveTime::from_hms(0, 0, 0)) + offset.to_chrono()
    }

    /// Construct a Stop object from a StopSchedule and a specific date
    pub fn from_stop_schedule(
        data: &'a RailroadData,
        stop: &StopSchedule,
        date: NaiveDate,
    ) -> Self {
        Stop {
            station: data.station(stop.station()).unwrap(),
            arrival: Self::inflate_stop_time(date, stop.arrival_offset()),
            departure: Self::inflate_stop_time(date, stop.departure_offset()),
        }
    }

    pub fn station(&self) -> &Station {
        self.station
    }

    pub fn arrival(&self) -> NaiveDateTime {
        self.arrival
    }

    pub fn departure(&self) -> NaiveDateTime {
        self.departure
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct Singularity<'a> {
    station: &'a Station,
    time: NaiveDateTime,
    train: Option<&'a Train>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Action<'a> {
    Wait(Duration),
    TrainWaits(&'a Train, Stop<'a>),
    Ride(&'a Train, Stop<'a>, Stop<'a>),
    Board(&'a Train),
    Unboard,
}

impl<'a> graph::Weight for Action<'a> {
    fn weight(&self) -> i64 {
        match self {
            Action::Wait(time) => time.num_seconds(),
            Action::TrainWaits(_, stop) => (stop.departure() - stop.arrival()).num_seconds(),
            // This minimizes train movements, to prevent e.g. going a->b->c->d->c->b instead of a->b->c->b, if they have the same dest time
            Action::Ride(_, start, end) => (end.arrival() - start.departure()).num_seconds() + 1,
            // This minimizes train switches
            Action::Board(_) => 60,
            Action::Unboard => 60,
        }
    }
}

type RailroadGraph<'a> = graph::Graph<Singularity<'a>, Action<'a>>;

impl<'a> RailroadGraph<'a> {
    fn from_data(
        data: &'a RailroadData,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Self {
        let mut result = Self::new();
        let mut stations_general: HashMap<&Station, HashSet<Singularity>> = HashMap::new();
        let first_possible_date = start_time.date();
        let last_possible_date = if end_time.time() == NaiveTime::from_hms(0, 0, 0) {
            end_time.date().pred()
        } else {
            end_time.date()
        };
        // Iterate all trains on all dates
        for train in data.trains() {
            // This is a preliminary filter, using dates only - we will do a fine-tuned filtering that includes time soon
            for date in train
                .dates()
                .filter(|&x| x >= &first_possible_date && x <= &last_possible_date)
            {
                let mut prev = None;
                for stop in train.stops() {
                    let stop = Stop::from_stop_schedule(data, stop, *date);
                    // Filter out all irrelevant stops
                    if stop.arrival > end_time || stop.departure < start_time {
                        continue;
                    }
                    // Make sure we have a singularity set for this station
                    if !stations_general.contains_key(stop.station) {
                        stations_general.insert(stop.station, HashSet::new());
                    }
                    let station_set = stations_general.get_mut(stop.station).unwrap();

                    // Create nodes for train arrival time and station time, and connect unboarding option
                    let arrival = Singularity {
                        station: stop.station,
                        time: stop.arrival,
                        train: Some(train),
                    };
                    let arrival_station = Singularity {
                        station: arrival.station,
                        time: arrival.time,
                        train: None,
                    };
                    result
                        .get_or_insert(&arrival)
                        .connect(Action::Unboard, arrival_station);
                    result.get_or_insert(&arrival_station);
                    station_set.insert(arrival_station);

                    // Connect previous stop
                    if let Some((prev_node, prev_stop)) = prev {
                        result
                            .get_mut(&prev_node)
                            .unwrap()
                            .connect(Action::Ride(train, prev_stop, stop), arrival);
                    }

                    // Handle waiting on train
                    // Create nodes for train departure time and station time if train arrival != departure
                    let (departure, departure_station) = if stop.arrival == stop.departure {
                        (arrival, arrival_station)
                    } else {
                        let departure = Singularity {
                            station: stop.station,
                            time: stop.departure(),
                            train: Some(train),
                        };
                        let departure_station = Singularity {
                            station: departure.station,
                            time: departure.time,
                            train: None,
                        };
                        result.get_or_insert(&departure);
                        station_set.insert(departure_station);

                        // Connect waiting on train edge (train waits in station)
                        result
                            .get_mut(&arrival)
                            .unwrap()
                            .connect(Action::TrainWaits(train, stop), departure);
                        (departure, departure_station)
                    };

                    // Connect boarding option
                    result
                        .get_or_insert(&departure_station)
                        .connect(Action::Board(train), departure);
                    prev = Some((departure, stop));
                }
            }
        }

        // Connect each station's singularities with wait edges
        for (_, station_set) in stations_general {
            let mut station_vec: Vec<Singularity> = station_set.into_iter().collect();
            station_vec.sort_unstable_by_key(|s| s.time);
            let mut prev = None;
            for curr in station_vec {
                if let Some(prev) = prev {
                    result
                        .get_mut(&prev)
                        .unwrap()
                        .connect(Action::Wait(curr.time - prev.time), curr);
                }
                prev = Some(curr);
            }
        }

        result
    }

    fn ensure(&mut self, s: Singularity<'a>) {
        if self.get(&s).is_none() {
            self.get_or_insert(&s);
            if let Some(next) = self
                .nodes()
                .map(|n| n.id())
                .filter(|n| n.train == s.train && n.station == s.station && n.time > s.time)
                .min_by_key(|n| n.time)
                .copied()
            {
                self.get_mut(&s)
                    .unwrap()
                    .connect(Action::Wait(next.time - s.time), next);
            }
            if let Some(prev) = self
                .nodes()
                .map(|n| n.id())
                .filter(|n| n.train == s.train && n.station == s.station && n.time < s.time)
                .max_by_key(|n| n.time)
                .copied()
            {
                self.get_mut(&prev)
                    .unwrap()
                    .connect(Action::Wait(s.time - prev.time), s);
            }
        }
    }
}

/// Holds information regarding a single train ride
pub struct RoutePart<'a> {
    train: &'a Train,
    start: Stop<'a>,
    end: Stop<'a>,
}

impl<'a> RoutePart<'a> {
    /// Create a new RoutePart object
    pub fn new(train: &'a Train, start: Stop<'a>, end: Stop<'a>) -> Self {
        RoutePart { train, start, end }
    }

    /// The train associated with the RoutePart object
    pub fn train(&self) -> &Train {
        self.train
    }

    /// The stop at which the train is boarded
    pub fn start(&self) -> Stop {
        self.start
    }

    /// The stop at which the train is unboarded
    pub fn end(&self) -> Stop {
        self.end
    }
}

impl<'a> fmt::Display for RoutePart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) -> {} ({})",
            self.start.station().name().to_owned(),
            self.start.departure(),
            self.end.station().name().to_owned(),
            self.end.arrival()
        )
    }
}

impl<'a> JSON for RoutePart<'a> {
    fn to_json(&self) -> JsonValue {
        let departure = DateTime::<Utc>::from_utc(self.start.departure(), Utc);
        let arrival = DateTime::<Utc>::from_utc(self.end.arrival(), Utc);
        object! {
            train: self.train.id().to_owned(),
            start_time: departure.to_rfc3339(),
            start_station: self.start.station().id(),
            end_time: arrival.to_rfc3339(),
            end_station: self.end.station().id()
        }
    }
}

/// Holds details of a route between stations
pub struct Route<'a> {
    parts: Vec<RoutePart<'a>>,
}

impl<'a> Route<'a> {
    /// Create a new Route object
    pub fn new() -> Self {
        Route { parts: Vec::new() }
    }

    /// Create a enw Route object from parts
    pub fn from_parts(parts: Vec<RoutePart<'a>>) -> Self {
        Route { parts }
    }

    /// Iterate over the parts of the route. Each RoutePart corresponds to a single train ride.
    pub fn parts(&self) -> impl Iterator<Item = &RoutePart> {
        self.parts.iter()
    }
}

impl<'a> fmt::Display for Route<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in self.parts.iter() {
            writeln!(f, "{}", part)?;
        }
        Ok(())
    }
}

impl<'a> JSON for Route<'a> {
    fn to_json(&self) -> JsonValue {
        let mut result = JsonValue::new_array();
        for part in &self.parts {
            result.push(part.to_json()).unwrap();
        }
        object! {
            parts: result
        }
    }
}

fn build_route<'a>(path: Vec<(Action<'a>, Singularity)>) -> Route<'a> {
    let mut route = Route::new();
    let mut last_train: Option<&Train> = None;
    let mut last_train_start: Option<Stop> = None;
    let mut last_train_end: Option<Stop> = None;
    for (action, _) in path {
        match action {
            Action::Wait(_) => {}
            Action::TrainWaits(_, _) => {}
            Action::Ride(train, start, end) => {
                match last_train {
                    Some(x) => assert!(x == train),
                    None => {
                        last_train = Some(train);
                        last_train_start = Some(start);
                    }
                }
                last_train_end = Some(end);
            }
            Action::Board(_) => {}
            Action::Unboard => {
                route.parts.push(RoutePart::new(
                    last_train.take().unwrap(),
                    last_train_start.take().unwrap(),
                    last_train_end.take().unwrap(),
                ));
            }
        }
    }
    route
}

/// Finds the single best route from the source to the destination station at the given time.
///
/// This obtains the route with the fastest arrival time, relative to the given time.
/// If more than one route is present, routes are prioritized according to least train switches, and least stations passed through in general.
/// The supplied end time is the latest possible arrival time that will be considered. This is used for optimization purposes.
pub fn get_best_single_route<'a>(
    data: &'a RailroadData,
    start_time: NaiveDateTime,
    start_station: &'a Station,
    end_time: NaiveDateTime,
    end_station: &'a Station,
) -> Option<Route<'a>> {
    let mut g = RailroadGraph::from_data(data, start_time, end_time);
    let origin = Singularity {
        station: start_station,
        time: start_time,
        train: None,
    };
    g.ensure(origin);
    let path = g.find_shortest_path(&origin, |s| s.station == end_station && s.train.is_none())?;
    Some(build_route(path))
}

/// Finds a route that arrives no later than the best route, but leaves as late as possible.
///
/// This obtains the route with the fastest arrival time, relative to the given time.
/// If more than one route is present, routes are prioritized according to latest departure time.
/// If still more than one route is present, routes are subsequently prioritized by least train switches, and least stations passed through in general.
/// The supplied end time is the latest possible arrival time that will be considered. This is used for optimization purposes.
pub fn get_latest_good_single_route<'a>(
    data: &'a RailroadData,
    start_time: NaiveDateTime,
    start_station: &'a Station,
    end_time: NaiveDateTime,
    end_station: &'a Station,
) -> Option<Route<'a>> {
    let mut g = RailroadGraph::from_data(data, start_time, end_time);
    let origin = Singularity {
        station: start_station,
        time: start_time,
        train: None,
    };
    g.ensure(origin);
    let path = g.find_shortest_path(&origin, |s| s.station == end_station && s.train.is_none())?;
    let mut route = build_route(path);
    let best_arrival = match route.parts().last() {
        Some(x) => x.end.arrival(),
        None => return Some(route),
    };
    while route.parts().last().unwrap().end.arrival() == best_arrival {
        let origin = Singularity {
            station: start_station,
            time: route.parts().next().unwrap().start.departure() + Duration::seconds(1),
            train: None,
        };
        g.ensure(origin);
        let path_opt =
            g.find_shortest_path(&origin, |s| s.station == end_station && s.train.is_none());
        route = match path_opt {
            Some(p) => build_route(p),
            None => break,
        };
    }
    Some(route)
}

/// Finds all good routes to the destination
///
/// This obtains all routes that have no better routes for the same arrival time.
/// The route search is started from start_time, and will not find routes ending later than end_time.
pub fn get_multiple_routes<'a>(
    data: &'a RailroadData,
    start_time: NaiveDateTime,
    start_station: &'a Station,
    end_time: NaiveDateTime,
    end_station: &'a Station,
) -> Vec<Route<'a>> {
    let mut g = RailroadGraph::from_data(data, start_time, end_time);
    let mut result = Vec::new();

    let origin = Singularity {
        station: start_station,
        time: start_time,
        train: None,
    };
    g.ensure(origin);
    let mut path_opt =
        g.find_shortest_path(&origin, |s| s.station == end_station && s.train.is_none());
    while let Some(path) = path_opt {
        let route = build_route(path);
        if route.parts.len() == 0 {
            result.push(route);
            break;
        }
        let origin = Singularity {
            station: start_station,
            time: route.parts().next().unwrap().start.departure() + Duration::seconds(1),
            train: None,
        };
        result.push(route);
        g.ensure(origin);
        path_opt = g.find_shortest_path(&origin, |s| s.station == end_station && s.train.is_none());
    }
    result
}
