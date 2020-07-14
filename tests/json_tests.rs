/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod test_data;
use harail::gtfs::{RailroadData, Station, Stop, Train};
use harail::{Route, RoutePart, JSON};
use test_data::time;

#[test]
fn stations_save() {
    let x = Station::new(100, "stationary");
    assert_eq!(x.to_json().dump(), r#"{"id":100,"name":"stationary"}"#);
}

#[test]
fn route_save() {
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(200, time(10, 30, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let train = data.train("1");
    let route = Route::from_parts(vec![RoutePart::new(
        train,
        train.stops().collect::<Vec<_>>()[0],
        train.stops().collect::<Vec<_>>()[1],
    )]);
    assert_eq!(
        route.to_json().dump(),
        r#"{"parts":[{"train":"1","start_time":"2000-01-01T10:00:00+00:00","start_station":100,"end_time":"2000-01-01T10:30:00+00:00","end_station":200}]}"#
    );
}
