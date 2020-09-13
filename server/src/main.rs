/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate rocket;

use bincode::deserialize_from;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use clap::{App, Arg};
use harail::{RailroadData, StationId, Stop, JSON};
use json::JsonValue;
use rocket::http::RawStr;
use rocket::request::{Form, FromFormValue, FromParam};
use rocket::response::{content, status};
use rocket::State;
use std::{error::Error, fs::File, io::BufReader, path::Path};

#[cfg(test)]
mod tests;

#[get("/stations")]
fn list_stations(data: State<RailroadData>) -> content::Json<String> {
    let json = JsonValue::Array(data.stations().map(|s| s.to_json()).collect());
    content::Json(json.dump())
}

struct HaDate(NaiveDate);

impl<'v> FromParam<'v> for HaDate {
    type Error = &'v RawStr;

    fn from_param(param: &'v RawStr) -> Result<Self, Self::Error> {
        let dt = param.parse::<DateTime<Utc>>().map_err(|_| param)?;
        Ok(HaDate(dt.naive_utc().date()))
    }
}

#[get("/trains/<id>/stops/<date>")]
fn get_train(data: State<RailroadData>, id: String, date: HaDate) -> Option<content::Json<String>> {
    let train = data.train(&id)?;
    let json = JsonValue::Array(
        train
            .stops()
            .map(|s| Stop::from_stop_schedule(&data, s, date.0).to_json())
            .collect(),
    );
    Some(content::Json(json.dump()))
}

#[derive(FromFormValue)]
enum SearchType {
    Best,
    Latest,
    Multi,
}

struct HaDateTime(NaiveDateTime);

impl<'v> FromFormValue<'v> for HaDateTime {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        let dt = form_value
            .parse::<DateTime<Utc>>()
            .map_err(|_| form_value)?;
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
    data: State<RailroadData>,
    options: Form<FindOptions>,
) -> Result<content::Json<String>, status::NotFound<String>> {
    let start_station = data
        .station(options.start_station)
        .ok_or_else(|| status::NotFound(String::from("start station not found")))?;
    let start_time = options.start_time.0;
    let end_station = data
        .station(options.end_station)
        .ok_or_else(|| status::NotFound(String::from("end station not found")))?;
    let end_time = options.end_time.0;
    Ok(content::Json(match options.search {
        SearchType::Best => {
            harail::get_best_single_route(&data, start_time, start_station, end_time, end_station)
                .ok_or_else(|| status::NotFound(String::from("no possible route found")))?
                .to_json()
                .dump()
        }
        SearchType::Latest => harail::get_latest_good_single_route(
            &data,
            start_time,
            start_station,
            end_time,
            end_station,
        )
        .ok_or_else(|| status::NotFound(String::from("no possible route found")))?
        .to_json()
        .dump(),
        SearchType::Multi => JsonValue::Array(
            harail::get_multiple_routes(&data, start_time, start_station, end_time, end_station)
                .into_iter()
                .map(|r| r.to_json())
                .collect(),
        )
        .dump(),
    }))
}

fn rocket(data: RailroadData) -> rocket::Rocket {
    rocket::ignite()
        .manage(data)
        .mount("/harail", routes![list_stations, get_train, find_route])
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("HaRail Server")
        .version("0.1")
        .author("Yuval Deutscher")
        .about("Because the Israel Railways app sucks™ (server edition)")
        .arg(
            Arg::with_name("DATABASE")
                .help("The HaRail database to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = Path::new(matches.value_of("DATABASE").unwrap());
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: RailroadData = deserialize_from(reader)?;

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(rocket(data).launch())?;
    Ok(())
}
