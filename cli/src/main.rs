/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bincode::{deserialize_from, serialize_into};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{Arg, Command};
use harail::{HaError, RailroadData, JSON};
use jzon::JsonValue;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const JSON_SPACES: u16 = 4;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("HaRail")
        .version("1.0.2")
        .author("Yuval Deutscher")
        .about("Because the Israel Railways app sucks™")
        .arg(
            Arg::new("DATABASE")
                .help("The HaRail database to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .help("Output in JSON format"),
        )
        .subcommand(Command::new("list-stations").about("Lists all stations"))
        .subcommand(Command::new("list-trains").about("Lists all trains"))
        .subcommand(
            Command::new("find")
                .about("Find paths between stations")
                .arg(
                    Arg::new("START_STATION")
                        .help("The ID of the starting station")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::new("DEST_STATION")
                        .help("The ID of the destination station")
                        .index(2)
                        .required(true),
                )
                .arg(
                    Arg::new("date")
                        .short('d')
                        .long("date")
                        .value_name("DATE")
                        .help("Specify date in DD/MM/YYYY format (default: today)"),
                )
                .arg(
                    Arg::new("time")
                        .short('t')
                        .long("time")
                        .value_name("TIME")
                        .help("Specify time in HH:MM:SS format (default: midnight)"),
                )
                .arg(
                    Arg::new("length")
                        .short('l')
                        .long("length")
                        .value_name("LENGTH")
                        .help("Specify length, in days, of the time period to search in (default: 1 day)"),
                )
                .arg(
                    Arg::new("delayed-leave")
                        .short('D')
                        .long("delayed-leave")
                        .help("Attempt to delay leaving time if destination time is not impacted"),
                )
                .arg(
                    Arg::new("multiple")
                        .short('m')
                        .long("multiple")
                        .help("Show multiple train options"),
                ),
        )
        .subcommand(
            Command::new("parse-gtfs")
                .about("Parse a GTFS database")
                .arg(
                    Arg::new("GTFS_PATH")
                        .help("The GTFS database to parse, in zip file or directory form")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("date-info")
                .about("Print information regarding the database start and expiration dates"),
        )
        .get_matches();

    let path = Path::new(matches.get_one::<String>("DATABASE").unwrap());

    if let Some(matches) = matches.subcommand_matches("parse-gtfs") {
        let gtfs_path = Path::new(matches.get_one::<String>("GTFS_PATH").unwrap());
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
    if matches.subcommand_matches("list-stations").is_some() {
        let mut stations: Vec<_> = data.stations().collect();
        stations.sort_by_key(|s| s.id());
        if matches.contains_id("json") {
            let json = JsonValue::Array(stations.into_iter().map(|s| s.to_json()).collect());
            println!("{}", json.pretty(JSON_SPACES));
        } else {
            stations.into_iter().for_each(|s| println!("{}", s));
        }
        return Ok(());
    }

    if matches.subcommand_matches("list-trains").is_some() {
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

    if matches.subcommand_matches("date-info").is_some() {
        let db_start = data
            .start_date()
            .ok_or_else(|| HaError::UsageError("Empty database".to_owned()))?;
        let db_end = data
            .end_date()
            .ok_or_else(|| HaError::UsageError("Empty database".to_owned()))?;
        println!("{} - {}", db_start, db_end);
        return Ok(());
    }

    if let Some(find_matches) = matches.subcommand_matches("find") {
        let start_time = NaiveDateTime::new(
            if let Some(date) = find_matches.get_one::<String>("date") {
                NaiveDate::parse_from_str(date, "%d/%m/%Y")
                    .map_err(|_| HaError::UsageError("Failed to parse date".to_owned()))?
            } else {
                chrono::Local::now().date_naive()
            },
            if let Some(time) = find_matches.get_one::<String>("time") {
                NaiveTime::parse_from_str(time, "%H:%M:%S")
                    .map_err(|_| HaError::UsageError("Failed to parse time".to_owned()))?
            } else {
                NaiveTime::from_hms_opt(0, 0, 0).unwrap()
            },
        );
        let n_days = find_matches
            .get_one::<String>("length")
            .map_or_else(|| Ok(1), |x| x.parse())
            .map_err(|_| HaError::UsageError("Failed to parse length".to_owned()))?;
        let end_time = start_time + chrono::Duration::days(n_days);
        let start_station = data
            .find_station(find_matches.get_one::<String>("START_STATION").unwrap())
            .ok_or_else(|| HaError::UsageError("Could not find source station".to_owned()))?;
        let end_station = data
            .find_station(find_matches.get_one::<String>("DEST_STATION").unwrap())
            .ok_or_else(|| HaError::UsageError("Could not find dest station".to_owned()))?;
        let routes = if find_matches.contains_id("multiple") {
            harail::get_multiple_routes(&data, start_time, start_station, end_time, end_station)
        } else if find_matches.contains_id("delayed-leave") {
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
        if matches.contains_id("json") {
            let json = JsonValue::Array(routes.into_iter().map(|r| r.to_json()).collect());
            println!("{}", json.pretty(JSON_SPACES));
        } else {
            routes.into_iter().for_each(|r| println!("{}", r));
        }
        return Ok(());
    }

    Err(Box::new(HaError::UsageError(
        "No operation specified".to_owned(),
    )))
}
