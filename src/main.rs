/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{App, Arg, SubCommand};
use std::path::Path;

fn main() {
    let matches = App::new("HaRail")
        .version("0.1")
        .author("Yuval Deutscher")
        .about("Because the Israel Railways app sucks™")
        .arg(
            Arg::with_name("DATABASE")
                .help("The input database to use")
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
    let data = harail::gtfs::RailroadData::from_gtfs(path, (start_time, end_time))
        .expect("Could not load GTFS database");
    if let Some(_) = matches.subcommand_matches("list-stations") {
        let mut stations: Vec<_> = data.stations().collect();
        stations.sort_by_key(|s| s.id());
        if matches.is_present("json") {
            panic!("Not implemented yet");
        } else {
            stations.into_iter().for_each(|s| println!("{}", s));
        }
        return;
    } else if let Some(matches) = matches.subcommand_matches("find") {
        let start_station = data
            .find_station(
                matches
                    .value_of("START_STATION")
                    .expect("Start station missing"),
            )
            .expect("Could not find source station");
        let end_station = data
            .find_station(
                matches
                    .value_of("DEST_STATION")
                    .expect("Destination station missing"),
            )
            .expect("Could not find dest station");
        let route = if matches.is_present("multiple") {
            panic!("Not implemented yet");
        } else if matches.is_present("delayed") {
            harail::get_latest_good_single_route(&data, start_time, start_station, end_station)
        } else {
            harail::get_best_single_route(&data, start_time, start_station, end_station)
        }
        .expect("Could not find best route");
        if matches.is_present("json") {
            panic!("Not implemented yet");
        } else {
            print!("{}", route);
        }
    }
}
