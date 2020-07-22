/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod test_data;
use chrono::{NaiveDateTime, NaiveTime};
use harail::{HaDuration, RailroadData, StopSchedule, Train};
use test_data::test_date;

#[test]
fn graph_time_cutoff() {
    let mut trains = Vec::new();
    trains.push(Train::from_stops_dates(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
        ],
        vec![test_date(), test_date().succ()],
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date().succ(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date().succ(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date().succ(), NaiveTime::from_hms(12, 00, 00)),
        data.station(400),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date().succ(), NaiveTime::from_hms(00, 00, 00)),
        data.station(400),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(11, 29, 59)),
        data.station(400),
    );
    assert!(route.is_none());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(11, 29, 59)),
        data.station(300),
    );
    assert!(route.is_some());
}