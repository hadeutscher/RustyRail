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
        vec![test_date(), test_date().succ_opt().unwrap()],
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(10, 00, 00).unwrap()),
        data.station(100).unwrap(),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(12, 00, 00).unwrap()),
        data.station(300).unwrap(),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(
            test_date().succ_opt().unwrap(),
            NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        ),
        data.station(100).unwrap(),
        NaiveDateTime::new(
            test_date().succ_opt().unwrap(),
            NaiveTime::from_hms_opt(12, 00, 00).unwrap(),
        ),
        data.station(300).unwrap(),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(10, 00, 00).unwrap()),
        data.station(100).unwrap(),
        NaiveDateTime::new(
            test_date().succ_opt().unwrap(),
            NaiveTime::from_hms_opt(12, 00, 00).unwrap(),
        ),
        data.station(400).unwrap(),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(10, 00, 00).unwrap()),
        data.station(100).unwrap(),
        NaiveDateTime::new(
            test_date().succ_opt().unwrap(),
            NaiveTime::from_hms_opt(00, 00, 00).unwrap(),
        ),
        data.station(400).unwrap(),
    );
    assert!(route.is_some());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(10, 00, 00).unwrap()),
        data.station(100).unwrap(),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(11, 29, 59).unwrap()),
        data.station(400).unwrap(),
    );
    assert!(route.is_none());
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(10, 00, 00).unwrap()),
        data.station(100).unwrap(),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms_opt(11, 29, 59).unwrap()),
        data.station(300).unwrap(),
    );
    assert!(route.is_some());
}
