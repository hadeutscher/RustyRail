/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod test_data;
use harail::{
    HaDuration, RailroadData, Route, RoutePart, Station, Stop, StopSchedule, Train, JSON,
};
use test_data::test_date;

#[test]
fn stations_save() {
    let x = Station::new(100, "stationary");
    assert_eq!(x.to_json().dump(), r#"{"id":100,"name":"stationary"}"#);
}

#[test]
fn route_save() {
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let train = data.train("1").unwrap();
    let stops = train
        .stops()
        .map(|s| Stop::from_stop_schedule(&data, s, test_date()))
        .collect::<Vec<_>>();
    let route = Route::from_parts(vec![RoutePart::new(train, stops[0], stops[1])]);
    assert_eq!(
        route.to_json().dump(),
        r#"{"parts":[{"train":"1","start_time":"2000-01-01T10:00:00+00:00","start_station":100,"end_time":"2000-01-01T10:30:00+00:00","end_station":200}]}"#
    );
}
