/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use harail::RailroadData;
use std::path::Path;

fn database_load(c: &mut Criterion) {
    c.bench_function("small GTFS load", |b| {
        b.iter(|| {
            RailroadData::from_gtfs_zip(Path::new(black_box(
                "fixtures/israel-public-transportation-min.zip",
            )))
            .unwrap()
        })
    });
}

criterion_group!(benches, database_load);
criterion_main!(benches);
