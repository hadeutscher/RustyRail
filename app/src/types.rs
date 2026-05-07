/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Shared data-transfer objects compiled for *both* the WASM client and
//! the native server.  All types must implement [`serde::Serialize`] /
//! [`serde::Deserialize`] and [`Clone`] / [`PartialEq`] so they work as
//! Dioxus component props and server-function return values.

use serde::{Deserialize, Serialize};

/// A transit station (id + display name).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationDto {
    pub id: u64,
    pub name: String,
}

/// One leg of a journey (a single train ride between two stations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutePartDto {
    /// Train identifier.
    pub train: String,
    /// Departure station id.
    pub start_station: u64,
    /// Arrival station id.
    pub end_station: u64,
    /// Departure time as an RFC 3339 string (UTC).
    pub start_time: String,
    /// Arrival time as an RFC 3339 string (UTC).
    pub end_time: String,
}

/// A complete journey composed of one or more [`RoutePartDto`] legs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDto {
    pub parts: Vec<RoutePartDto>,
}

/// One stop along a train's full schedule (station + scheduled times as offsets from midnight).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainStopDto {
    /// Station identifier.
    pub station_id: u64,
    /// Human-readable station name.
    pub station_name: String,
    /// Scheduled arrival time, formatted as `"HH:MM"`.
    pub arrival_offset: String,
    /// Scheduled departure time, formatted as `"HH:MM"`.
    pub departure_offset: String,
}
