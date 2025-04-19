/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use harail::Station;

use chrono::NaiveDate;

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
