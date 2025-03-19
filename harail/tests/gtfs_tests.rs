/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use harail::RailroadData;
use std::path::Path;

#[test]
fn load_zipped_gtfs() {
    let data =
        RailroadData::from_gtfs_zip(Path::new("fixtures/israel-public-transportation-min.zip"))
            .unwrap();
    assert_eq!(data.stations().count(), 66);
    assert_eq!(data.trains().count(), 4119);
}
