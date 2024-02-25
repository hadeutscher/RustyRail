/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::rocket;
use chrono::NaiveDate;
use harail::{HaDuration, RailroadData, Station, StopSchedule, Train};
use rocket::http::Status;
use rocket::local::blocking::Client;

pub fn stations() -> Vec<Station> {
    vec![
        Station::new(100, "stat_a"),
        Station::new(200, "stat_b"),
        Station::new(300, "stat_c"),
        Station::new(400, "stat_d"),
        Station::new(500, "stat_e"),
        Station::new(600, "stat_f"),
    ]
}

pub fn test_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
}

#[test]
fn stations_list() {
    let data = RailroadData::from_stations_trains(stations(), vec![]);
    let client = Client::tracked(rocket(data, None)).expect("valid rocket instance");
    let response = client.get("/harail/stations").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let json = jzon::parse(&response.into_string().unwrap()).unwrap();
    assert_eq!(json.len(), 6);
    for json in json.members() {
        let id = json["id"].as_u64().unwrap();
        let name = json["name"].as_str().unwrap();
        assert_eq!(
            stations()
                .into_iter()
                .find(|x| x.id() == id)
                .unwrap()
                .name(),
            name
        );
    }
}

#[test]
fn train_stops() {
    let trains = vec![Train::from_stops_dates(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
        ],
        vec![test_date(), test_date().succ_opt().unwrap()],
    )];
    let data = RailroadData::from_stations_trains(stations(), trains);
    let client = Client::tracked(rocket(data, None)).expect("valid rocket instance");
    let response = client
        .get("/harail/trains/1/stops/2000-01-01T00:00:00Z")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string(),
        Some(String::from(
            r#"[{"station":100,"arrival":"2000-01-01T10:00:00+00:00","departure":"2000-01-01T10:00:00+00:00"},{"station":200,"arrival":"2000-01-01T10:30:00+00:00","departure":"2000-01-01T10:30:00+00:00"},{"station":300,"arrival":"2000-01-01T11:00:00+00:00","departure":"2000-01-01T11:00:00+00:00"},{"station":400,"arrival":"2000-01-01T11:30:00+00:00","departure":"2000-01-01T11:30:00+00:00"}]"#
        ))
    );
}

#[test]
fn find_routes() {
    let trains = vec![Train::from_stops_dates(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
        ],
        vec![test_date(), test_date().succ_opt().unwrap()],
    )];
    let data = RailroadData::from_stations_trains(stations(), trains);
    let client = Client::tracked(rocket(data, None)).expect("valid rocket instance");
    let response = client
        .get("/harail/routes/find?search=best&start_station=100&start_time=2000-01-01T00:00:00Z&end_station=400&end_time=2000-01-02T00:00:00Z")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string(),
        Some(String::from(
            r#"{"parts":[{"train":"1","start_time":"2000-01-01T10:00:00+00:00","start_station":100,"end_time":"2000-01-01T11:30:00+00:00","end_station":400}]}"#
        ))
    );
}
