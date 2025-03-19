/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
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

fn graph_building(c: &mut Criterion) {
    let data = RailroadData::from_gtfs_zip(Path::new(black_box(
        "fixtures/israel-public-transportation-min.zip",
    )))
    .unwrap();
    let start_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 9).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );
    let end_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 10).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );

    // Use same value for start and end station to minimize the cost of Dijkstra's algorithm
    let station = data.stations().next().unwrap();

    c.bench_function("1 day graph building", |b| {
        b.iter(|| {
            harail::get_best_single_route(
                black_box(&data),
                black_box(start_time),
                black_box(station),
                black_box(end_time),
                black_box(station),
            )
            .unwrap()
        })
    });

    let end_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 19).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );

    c.bench_function("10 day graph building", |b| {
        b.iter(|| {
            harail::get_best_single_route(
                black_box(&data),
                black_box(start_time),
                black_box(station),
                black_box(end_time),
                black_box(station),
            )
            .unwrap()
        })
    });
}

fn graph_processing(c: &mut Criterion) {
    let data = RailroadData::from_gtfs_zip(Path::new(black_box(
        "fixtures/israel-public-transportation-min.zip",
    )))
    .unwrap();
    let start_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 9).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );
    let end_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 10).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );

    let start_station = data.station(37382).unwrap();
    let end_station = data.station(37382).unwrap();

    c.bench_function("1 day path finding", |b| {
        b.iter(|| {
            harail::get_multiple_routes(
                black_box(&data),
                black_box(start_time),
                black_box(start_station),
                black_box(end_time),
                black_box(end_station),
            )
        })
    });

    let end_time = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2020, 9, 19).unwrap(),
        NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    );

    c.bench_function("10 day path finding", |b| {
        b.iter(|| {
            harail::get_multiple_routes(
                black_box(&data),
                black_box(start_time),
                black_box(start_station),
                black_box(end_time),
                black_box(end_station),
            )
        })
    });
}

criterion_group!(benches, database_load, graph_building, graph_processing);
criterion_main!(benches);
