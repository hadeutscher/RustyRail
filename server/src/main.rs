/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Required until this issue is fixed https://github.com/rwf2/Rocket/issues/2655
#![allow(clippy::blocks_in_conditions)]

#[macro_use]
extern crate rocket;

use bincode::config;
use bincode::serde::decode_from_std_read;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use clap::{Arg, Command};
use harail::{JSON, RailroadData, StationId, Stop};
use jzon::JsonValue;
use rocket::State;
use rocket::form::{self, FromFormField, ValueField};
use rocket::fs::FileServer;
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket::response::content::RawJson;
use rocket::response::status;
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests;

#[get("/stations")]
fn list_stations(data: &State<RailroadData>) -> RawJson<String> {
    let json = JsonValue::Array(data.stations().map(|s| s.to_json()).collect());
    RawJson(json.dump())
}

struct HaDate(NaiveDate);

impl<'v> FromParam<'v> for HaDate {
    type Error = &'v RawStr;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        let dt = param.parse::<DateTime<Utc>>().map_err(|_| param)?;
        Ok(HaDate(dt.naive_utc().date()))
    }
}

#[get("/trains/<id>/stops/<date>")]
fn get_train(data: &State<RailroadData>, id: &str, date: HaDate) -> Option<RawJson<String>> {
    let train = data.train(id)?;
    let json = JsonValue::Array(
        train
            .stops()
            .map(|s| Stop::from_stop_schedule(data, s, date.0).to_json())
            .collect(),
    );
    Some(RawJson(json.dump()))
}

#[derive(FromFormField)]
enum SearchType {
    Best,
    Latest,
    Multi,
}

struct HaDateTime(NaiveDateTime);

#[rocket::async_trait]
impl<'v> FromFormField<'v> for HaDateTime {
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        let dt = field.value.parse::<DateTime<Utc>>().map_err(|_| {
            form::Error::validation(format! {"Cannot parse {} as date", field.value})
        })?;
        Ok(HaDateTime(dt.naive_utc()))
    }
}

#[derive(FromForm)]
struct FindOptions {
    search: SearchType,
    start_station: StationId,
    start_time: HaDateTime,
    end_station: StationId,
    end_time: HaDateTime,
}

#[get("/routes/find?<options..>")]
fn find_route(
    data: &State<RailroadData>,
    options: FindOptions,
) -> Result<RawJson<String>, status::NotFound<String>> {
    let start_station = data
        .station(options.start_station)
        .ok_or_else(|| status::NotFound(String::from("start station not found")))?;
    let start_time = options.start_time.0;
    let end_station = data
        .station(options.end_station)
        .ok_or_else(|| status::NotFound(String::from("end station not found")))?;
    let end_time = options.end_time.0;
    Ok(RawJson(match options.search {
        SearchType::Best => {
            harail::get_best_single_route(data, start_time, start_station, end_time, end_station)
                .ok_or_else(|| status::NotFound(String::from("no possible route found")))?
                .to_json()
                .dump()
        }
        SearchType::Latest => harail::get_latest_good_single_route(
            data,
            start_time,
            start_station,
            end_time,
            end_station,
        )
        .ok_or_else(|| status::NotFound(String::from("no possible route found")))?
        .to_json()
        .dump(),
        SearchType::Multi => JsonValue::Array(
            harail::get_multiple_routes(data, start_time, start_station, end_time, end_station)
                .into_iter()
                .map(|r| r.to_json())
                .collect(),
        )
        .dump(),
    }))
}

fn rocket(data: RailroadData, static_path: Option<&Path>) -> rocket::Rocket<rocket::Build> {
    let rocket = rocket::build()
        .manage(data)
        .mount("/harail", routes![list_stations, get_train, find_route]);
    match static_path {
        Some(path) => rocket.mount("/", FileServer::from(path)),
        None => rocket,
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let matches = Command::new("HaRail Server")
        .version(VERSION.unwrap_or_default())
        .author("Yuval Deutscher")
        .about("Because the Israel Railways app sucks™ (server edition)")
        .arg(
            Arg::new("DATABASE")
                .help("The HaRail database to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("static")
                .short('s')
                .long("static")
                .value_name("STATIC")
                .help("Path to static assets (optional)"),
        )
        .get_matches();

    let static_path = matches.get_one::<String>("static").map(PathBuf::from);
    let path = Path::new(matches.get_one::<String>("DATABASE").unwrap());
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let data: RailroadData = decode_from_std_read(&mut reader, config::legacy()).unwrap();
    rocket(data, static_path.as_deref())
        .ignite()
        .await?
        .launch()
        .await?;
    Ok(())
}
