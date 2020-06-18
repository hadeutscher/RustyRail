/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{App, Arg};
use std::path::Path;

fn main() {
    let matches = App::new("HaRail")
        .arg(
            Arg::with_name("database")
                .help("Sets the input database to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let start_time = NaiveDateTime::new(
        NaiveDate::from_ymd(2020, 06, 22),
        NaiveTime::from_hms(0, 0, 0),
    );
    let end_time = NaiveDateTime::new(
        NaiveDate::from_ymd(2020, 06, 23),
        NaiveTime::from_hms(0, 0, 0),
    );
    let path = Path::new(matches.value_of("database").unwrap());
    let data = harail::gtfs::RailroadData::from_gtfs(path, (start_time, end_time))
        .expect("Could not load GTFS database");
    let start_station = data
        .find_station("נהריה")
        .expect("Could not find source station");
    let end_station = data
        .find_station("באר שבע מרכז")
        .expect("Could not find dest station");
    let route = harail::get_best_single_route(&data, start_time, start_station, end_station)
        .expect("Could not find best route");
    println!("{}", route);
}
