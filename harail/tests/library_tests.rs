/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod test_data;
use chrono::{Duration, NaiveDateTime, NaiveTime};
use harail::{RailroadData, StopSchedule, Train};
use test_data::test_date;

#[test]
fn graph_time_cutoff() {
    let trains = vec![Train::from_stops_dates(
        "1",
        vec![
            StopSchedule::new(100, Duration::hours(10), None),
            StopSchedule::new(200, Duration::hours(10) + Duration::minutes(30), None),
            StopSchedule::new(300, Duration::hours(11), None),
            StopSchedule::new(400, Duration::hours(11) + Duration::minutes(30), None),
        ],
        vec![test_date(), test_date().succ_opt().unwrap()],
    )];
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
