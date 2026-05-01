/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Unit tests for the business-logic helpers.
//!
//! These tests compile only with `--features server` because the helpers
//! depend on the `harail` crate, which is a server-only dependency.
//!
//! Run with:
//!   cargo test --features server -p harail-app

use super::{routes_from_data, stations_from_data};
use chrono::NaiveDate;
use harail::{HaDuration, RailroadData, Station, StopSchedule, Train};

// ── Fixtures ────────────────────────────────────────────────────────────────

pub fn fixture_stations() -> Vec<Station> {
    vec![
        Station::new(100, "stat_a"),
        Station::new(200, "stat_b"),
        Station::new(300, "stat_c"),
        Station::new(400, "stat_d"),
        Station::new(500, "stat_e"),
        Station::new(600, "stat_f"),
    ]
}

pub fn fixture_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn stations_list_returns_all_sorted() {
    let data = RailroadData::from_stations_trains(fixture_stations(), vec![]);
    let stations = stations_from_data(&data);

    assert_eq!(stations.len(), 6);
    // Verify alphabetical order.
    for w in stations.windows(2) {
        assert!(w[0].name <= w[1].name, "{} > {}", w[0].name, w[1].name);
    }
    // Verify each original station is present.
    for fixture in fixture_stations() {
        assert!(
            stations
                .iter()
                .any(|s| s.id == fixture.id() && s.name.as_str() == fixture.name()),
            "station {} missing",
            fixture.id()
        );
    }
}

#[test]
fn find_routes_single_train() {
    let trains = vec![Train::from_stops_dates(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 0, 0), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 0), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 0, 0), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 0), None),
        ],
        vec![fixture_date(), fixture_date().succ_opt().unwrap()],
    )];
    let data = RailroadData::from_stations_trains(fixture_stations(), trains);

    let routes = routes_from_data(
        &data,
        100,
        "2000-01-01T00:00:00Z",
        400,
        "2000-01-02T00:00:00Z",
    )
    .unwrap();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].parts.len(), 1);
    let part = &routes[0].parts[0];
    assert_eq!(part.train, "1");
    assert_eq!(part.start_station, 100);
    assert_eq!(part.end_station, 400);
    assert!(part.start_time.starts_with("2000-01-01T10:00:00"));
    assert!(part.end_time.starts_with("2000-01-01T11:30:00"));
}

#[test]
fn find_routes_unknown_station_returns_error() {
    let data = RailroadData::from_stations_trains(fixture_stations(), vec![]);
    let result = routes_from_data(
        &data,
        9999,
        "2000-01-01T00:00:00Z",
        100,
        "2000-01-02T00:00:00Z",
    );
    assert!(result.is_err());
}
