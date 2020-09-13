/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bincode::{deserialize_from, serialize_into};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{App, Arg, SubCommand};
use harail::{HaError, RailroadData, JSON};
use json::JsonValue;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const JSON_SPACES: u16 = 4;

fn main() -> Result<(), Box<dyn Error>> {
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
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Output in JSON format"),
        )
        .subcommand(SubCommand::with_name("list-stations").about("Lists all stations"))
        .subcommand(SubCommand::with_name("list-trains").about("Lists all trains"))
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
                    Arg::with_name("length")
                        .short("l")
                        .long("length")
                        .value_name("LENGTH")
                        .help("Specify length, in days, of the time period to search in (default: 1 day)")
                        .takes_value(true),
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
                        .help("The GTFS database to parse, in zip file or directory form")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("date-info")
                .about("Print information regarding the database start and expiration dates"),
        )
        .get_matches();

    let path = Path::new(matches.value_of("DATABASE").unwrap());

    if let Some(matches) = matches.subcommand_matches("parse-gtfs") {
        let gtfs_path = Path::new(matches.value_of("GTFS_PATH").unwrap());
        let load_result = if gtfs_path.is_dir() {
            RailroadData::from_gtfs_directory(gtfs_path)
        } else {
            RailroadData::from_gtfs_zip(gtfs_path)
        };
        let data = load_result
            .map_err(|_| HaError::UsageError("Could not load GTFS database".to_owned()))?;
        let file = File::create(path).map_err(|_| {
            HaError::UsageError("Could not open database file for writing".to_owned())
        })?;
        let writer = BufWriter::new(file);
        serialize_into(writer, &data)
            .map_err(|_| HaError::UsageError("Could not serialize database".to_owned()))?;
        return Ok(());
    }

    let file = File::open(path)
        .map_err(|_| HaError::UsageError("Could not open database file".to_owned()))?;
    let reader = BufReader::new(file);
    let data: RailroadData = deserialize_from(reader)
        .map_err(|_| HaError::UsageError("Could not deserialize database".to_owned()))?;
    if let Some(_) = matches.subcommand_matches("list-stations") {
        let mut stations: Vec<_> = data.stations().collect();
        stations.sort_by_key(|s| s.id());
        if matches.is_present("json") {
            let json = JsonValue::Array(stations.into_iter().map(|s| s.to_json()).collect());
            println!("{}", json.pretty(JSON_SPACES));
        } else {
            stations.into_iter().for_each(|s| println!("{}", s));
        }
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("list-trains") {
        let mut trains: Vec<_> = data.trains().collect();
        trains.sort_by_key(|t| t.id());
        trains.into_iter().for_each(|t| {
            println!(
                "{} : {} ({}) -> {} ({})",
                t.id(),
                t.stops().next().unwrap().station(),
                t.stops().next().unwrap().departure_offset(),
                t.stops().last().unwrap().station(),
                t.stops().last().unwrap().arrival_offset()
            )
        });
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("date-info") {
        let db_start = data
            .start_date()
            .ok_or_else(|| HaError::UsageError("Empty database".to_owned()))?;
        let db_end = data
            .end_date()
            .ok_or_else(|| HaError::UsageError("Empty database".to_owned()))?;
        println!("{} - {}", db_start, db_end);
        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("find") {
        let start_time = NaiveDateTime::new(
            if let Some(date) = matches.value_of("date") {
                NaiveDate::parse_from_str(date, "%d/%m/%Y")
                    .map_err(|_| HaError::UsageError("Failed to parse date".to_owned()))?
            } else {
                chrono::Local::now().date().naive_local()
            },
            if let Some(time) = matches.value_of("time") {
                NaiveTime::parse_from_str(time, "%H:%M:%S")
                    .map_err(|_| HaError::UsageError("Failed to parse time".to_owned()))?
            } else {
                NaiveTime::from_hms(0, 0, 0)
            },
        );
        let n_days = matches
            .value_of("length")
            .map_or_else(|| Ok(1), |x| x.parse())
            .map_err(|_| HaError::UsageError("Failed to parse length".to_owned()))?;
        let end_time = start_time + chrono::Duration::days(n_days);
        let start_station = data
            .find_station(matches.value_of("START_STATION").unwrap())
            .ok_or_else(|| HaError::UsageError("Could not find source station".to_owned()))?;
        let end_station = data
            .find_station(matches.value_of("DEST_STATION").unwrap())
            .ok_or_else(|| HaError::UsageError("Could not find dest station".to_owned()))?;
        let routes = if matches.is_present("multiple") {
            harail::get_multiple_routes(&data, start_time, start_station, end_time, end_station)
        } else if matches.is_present("delayed") {
            vec![harail::get_latest_good_single_route(
                &data,
                start_time,
                start_station,
                end_time,
                end_station,
            )
            .ok_or_else(|| HaError::UsageError("No such route".to_owned()))?]
        } else {
            vec![harail::get_best_single_route(
                &data,
                start_time,
                start_station,
                end_time,
                end_station,
            )
            .ok_or_else(|| HaError::UsageError("No such route".to_owned()))?]
        };
        if matches.is_present("json") {
            let json = JsonValue::Array(routes.into_iter().map(|r| r.to_json()).collect());
            println!("{}", json.pretty(JSON_SPACES));
        } else {
            routes.into_iter().for_each(|r| println!("{}", r));
        }
        return Ok(());
    }

    return Err(Box::new(HaError::UsageError(
        "No operation specified".to_owned(),
    )));
}
