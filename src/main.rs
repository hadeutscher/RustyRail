/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use(object)]
extern crate json;

use bincode::{deserialize_from, serialize_into};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{App, Arg, SubCommand};
use harail::{RailroadData, JSON};
use json::JsonValue;
use std::{fs::File, path::Path};

fn main() {
    let matches = App::new("HaRail")
        .version("0.1")
        .author("Yuval Deutscher")
        .about("Because the Israel Railways app sucks™")
        .arg(
            Arg::with_name("DATABASE")
                .help("The HaRail database to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("date")
                .short("d")
                .long("date")
                .value_name("DATE")
                .help("Specify date in DD/MM/YYYY format (default: today)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("time")
                .short("t")
                .long("time")
                .value_name("TIME")
                .help("Specify time in HH:MM:SS format (default: midnight)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Output in JSON format"),
        )
        .subcommand(SubCommand::with_name("list-stations").about("Lists all stations"))
        .subcommand(
            SubCommand::with_name("find")
                .about("Find paths between stations")
                .arg(
                    Arg::with_name("START_STATION")
                        .help("The ID of the starting station")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("DEST_STATION")
                        .help("The ID of the destination station")
                        .index(2)
                        .required(true),
                )
                .arg(
                    Arg::with_name("delayed-leave")
                        .short("D")
                        .long("delayed-leave")
                        .help("Attempt to delay leaving time if destination time is not impacted"),
                )
                .arg(
                    Arg::with_name("multiple")
                        .short("m")
                        .long("multiple")
                        .help("Show multiple train options"),
                ),
        )
        .subcommand(
            SubCommand::with_name("parse-gtfs")
                .about("Parse a GTFS database")
                .arg(
                    Arg::with_name("GTFS_PATH")
                        .help("The GTFS database to parse")
                        .index(1)
                        .required(true),
                ),
        )
        .get_matches();

    let start_time = NaiveDateTime::new(
        if let Some(date) = matches.value_of("date") {
            NaiveDate::parse_from_str(date, "%d/%m/%Y").expect("Failed to parse date")
        } else {
            chrono::Local::now().date().naive_local()
        },
        if let Some(time) = matches.value_of("time") {
            NaiveTime::parse_from_str(time, "%H:%M:%S").expect("Failed to parse time")
        } else {
            NaiveTime::from_hms(0, 0, 0)
        },
    );
    let end_time = start_time + chrono::Duration::days(1);
    let path = Path::new(matches.value_of("DATABASE").unwrap());

    if let Some(matches) = matches.subcommand_matches("parse-gtfs") {
        let gtfs_path = Path::new(matches.value_of("GTFS_PATH").unwrap());
        let data = RailroadData::from_gtfs(gtfs_path, (start_time, end_time))
            .expect("Could not load GTFS database");
        let file = File::create(path).expect("Could not open database file for writing");
        serialize_into(file, &data).expect("Could not serialize database");
        return;
    }

    let file = File::open(path).expect("Could not open database file");
    let data: RailroadData = deserialize_from(file).expect("Could not deserialize database");
    if let Some(_) = matches.subcommand_matches("list-stations") {
        let mut stations: Vec<_> = data.stations().collect();
        stations.sort_by_key(|s| s.id());
        if matches.is_present("json") {
            let mut json_stations = JsonValue::new_array();
            for station in stations {
                json_stations.push(station.to_json()).unwrap()
            }
            println!("{}", object! {stations: json_stations }.dump());
        } else {
            stations.into_iter().for_each(|s| println!("{}", s));
        }
        return;
    }

    if let Some(matches) = matches.subcommand_matches("find") {
        let start_station = data
            .find_station(matches.value_of("START_STATION").unwrap())
            .expect("Could not find source station");
        let end_station = data
            .find_station(matches.value_of("DEST_STATION").unwrap())
            .expect("Could not find dest station");
        let routes = if matches.is_present("multiple") {
            harail::get_multiple_routes(&data, start_time, start_station, end_station)
        } else if matches.is_present("delayed") {
            vec![harail::get_latest_good_single_route(
                &data,
                start_time,
                start_station,
                end_station,
            )
            .expect("Could not find best route")]
        } else {
            vec![
                harail::get_best_single_route(&data, start_time, start_station, end_station)
                    .expect("Could not find best route"),
            ]
        };
        if matches.is_present("json") {
            let mut json_routes = JsonValue::new_array();
            for route in routes {
                json_routes.push(route.to_json()).unwrap()
            }
            println!("{}", object! {routes: json_routes }.dump());
        } else {
            routes.into_iter().for_each(|r| println!("{}", r));
        }
        return;
    }
}
