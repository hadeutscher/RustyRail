/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use harail::gtfs::Station;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

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

pub fn time(h: u32, m: u32, s: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd(2000, 01, 01),
        NaiveTime::from_hms(h, m, s),
    )
}
