/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod types;

#[cfg(feature = "server")]
mod db;

#[cfg(all(test, feature = "server"))]
mod tests;

use chrono::{DateTime, FixedOffset};
use dioxus::prelude::*;
use dioxus_sdk::storage::{LocalStorage, use_storage};
use types::{RouteDto, RoutePartDto, StationDto, TrainStopDto};

// ---------------------------------------------------------------------------
// Server-only: shared state (global, set once at startup)
// ---------------------------------------------------------------------------

#[cfg(feature = "server")]
use db::SharedData;

/// Global database handle; initialised once in `main` before the server
/// begins accepting requests.
#[cfg(feature = "server")]
static RAILROAD_DATA: std::sync::OnceLock<SharedData> = std::sync::OnceLock::new();

// ---------------------------------------------------------------------------
// Server-only: pure business-logic helpers (also used by tests)
// ---------------------------------------------------------------------------

/// Returns all stations sorted alphabetically by name.
#[cfg(feature = "server")]
pub fn stations_from_data(data: &harail::RailroadData) -> Vec<StationDto> {
    let mut v: Vec<StationDto> = data
        .stations()
        .map(|s| StationDto {
            id: s.id(),
            name: s.name().to_owned(),
        })
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

/// Finds all good routes between two stations in the given time window,
/// returning them as serialisable DTOs.
#[cfg(feature = "server")]
pub fn routes_from_data(
    data: &harail::RailroadData,
    start_station_id: u64,
    start_time_str: &str,
    end_station_id: u64,
    end_time_str: &str,
) -> Result<Vec<RouteDto>, String> {
    use chrono::{DateTime, Utc};

    let start_st = data
        .station(start_station_id)
        .ok_or_else(|| "Start station not found".to_owned())?;
    let end_st = data
        .station(end_station_id)
        .ok_or_else(|| "End station not found".to_owned())?;

    let start_dt = start_time_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?
        .naive_utc();
    let end_dt = end_time_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?
        .naive_utc();

    let routes = harail::get_multiple_routes(data, start_dt, start_st, end_dt, end_st);

    let dtos = routes
        .into_iter()
        .map(|route| {
            let parts = route
                .parts()
                .map(|part| {
                    let dep =
                        DateTime::<Utc>::from_naive_utc_and_offset(part.start().departure(), Utc);
                    let arr = DateTime::<Utc>::from_naive_utc_and_offset(part.end().arrival(), Utc);
                    RoutePartDto {
                        train: part.train().id().to_owned(),
                        start_station: part.start().station().id(),
                        end_station: part.end().station().id(),
                        start_time: dep.to_rfc3339(),
                        end_time: arr.to_rfc3339(),
                    }
                })
                .collect();
            RouteDto { parts }
        })
        .collect();

    Ok(dtos)
}

/// Resolves every stop for the given train identifier, returning them in
/// schedule order as serialisable DTOs.
#[cfg(feature = "server")]
pub fn train_stops_from_data(
    data: &harail::RailroadData,
    train_id: &str,
) -> Result<Vec<TrainStopDto>, String> {
    let train = data
        .train(train_id)
        .ok_or_else(|| format!("Train '{train_id}' not found"))?;

    let stops = train
        .stops()
        .map(|stop| {
            let station_id = stop.station();
            let station_name = data
                .station(station_id)
                .map_or_else(|| "Unknown".to_owned(), |s| s.name().to_owned());

            let arr_secs = stop.arrival_offset().num_seconds() as u64;
            let dep_secs = stop.departure_offset().num_seconds() as u64;

            TrainStopDto {
                station_id,
                station_name,
                arrival_offset: format!("{:02}:{:02}", arr_secs / 3600, (arr_secs % 3600) / 60),
                departure_offset: format!("{:02}:{:02}", dep_secs / 3600, (dep_secs % 3600) / 60),
            }
        })
        .collect();

    Ok(stops)
}

// ---------------------------------------------------------------------------
// Server functions  (signature visible to WASM; body runs on the server)
// ---------------------------------------------------------------------------

/// Returns all stations, sorted alphabetically.
#[server(endpoint = "get_stations")]
pub async fn get_stations() -> Result<Vec<StationDto>, ServerFnError> {
    let guard = RAILROAD_DATA
        .get()
        .ok_or_else(|| ServerFnError::new("Database not initialised"))?
        .read()
        .await;
    Ok(stations_from_data(&guard))
}

/// Searches for all good routes in the given time window.
///
/// `start_time` and `end_time` must be RFC 3339 / ISO 8601 strings in UTC,
/// e.g. `"2025-01-01T09:00:00Z"`.
#[server(endpoint = "find_routes")]
pub async fn find_routes(
    start_station: u64,
    start_time: String,
    end_station: u64,
    end_time: String,
) -> Result<Vec<RouteDto>, ServerFnError> {
    let guard = RAILROAD_DATA
        .get()
        .ok_or_else(|| ServerFnError::new("Database not initialised"))?
        .read()
        .await;
    routes_from_data(&guard, start_station, &start_time, end_station, &end_time)
        .map_err(ServerFnError::new)
}

/// Returns all scheduled stops for the given train identifier, in order.
#[server(endpoint = "get_train_stops")]
pub async fn get_train_stops(train_id: String) -> Result<Vec<TrainStopDto>, ServerFnError> {
    let guard = RAILROAD_DATA
        .get()
        .ok_or_else(|| ServerFnError::new("Database not initialised"))?
        .read()
        .await;
    train_stops_from_data(&guard, &train_id).map_err(ServerFnError::new)
}

// ---------------------------------------------------------------------------
// Utility: format an RFC 3339 timestamp as "HH:MM"
// ---------------------------------------------------------------------------

/// Extracts the `HH:MM` portion from an RFC 3339 string without pulling in
/// `chrono` on the WASM target.
#[allow(unused)]
fn fmt_hhmm(iso: &str) -> String {
    // e.g. "2025-01-01T10:30:00+00:00"  →  "10:30"
    iso.find('T')
        .and_then(|t| iso.get(t + 1..t + 6))
        .unwrap_or(iso)
        .to_owned()
}

// ---------------------------------------------------------------------------
// Default date / time helpers
// ---------------------------------------------------------------------------

pub fn local_time() -> DateTime<FixedOffset> {
    #[cfg(target_arch = "wasm32")]
    {
        // Get JS Date (local time)
        let date = js_sys::Date::new_0();

        // Convert milliseconds → seconds + nanoseconds
        let millis = date.get_time();
        let secs = (millis / 1000.0) as i64;
        let nanos = ((millis % 1000.0) * 1_000_000.0) as u32;

        // Convert to UTC DateTime
        let utc = DateTime::<chrono::Utc>::from_timestamp(secs, nanos).expect("invalid timestamp");

        // Browser local offset (minutes)
        let offset_minutes = date.get_timezone_offset() as i32 * -1;
        let offset = FixedOffset::east_opt(offset_minutes * 60).expect("invalid offset");

        utc.with_timezone(&offset)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Local::now().fixed_offset()
    }
}

fn default_date_time() -> (String, String, String) {
    let now = local_time();
    let end = now + chrono::Duration::hours(3);
    let end_time = if end.date_naive() > now.date_naive() {
        "23:59".to_string()
    } else {
        end.format("%H:%M").to_string()
    };
    (
        now.format("%Y-%m-%d").to_string(),
        now.format("%H:%M").to_string(),
        end_time,
    )
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Root application component.
#[component]
fn App() -> Element {
    // use_server_future blocks SSR until the future resolves, then serialises
    // the result into the page so the client hydrates with data already
    // present — the None (loading) branch is never visible in practice.
    let stations_res = use_server_future(get_stations)?;

    rsx! {
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("/assets/main.css") }
        div { class: "container",
            h1 { "HaRail" }
            div { class: "card",
                match &*stations_res.read_unchecked() {
                    Some(Ok(stations)) => rsx! {
                        RouteFinder { stations: stations.clone() }
                    },
                    Some(Err(e)) => rsx! {
                        p { class: "error-text", "Error loading stations: {e}" }
                    },
                    None => rsx! {
                        p { class: "loading-text", "Loading stations…" }
                    },
                }
            }
        }
    }
}

/// Props for [`RouteFinder`].
#[derive(Props, Clone, PartialEq)]
struct RouteFinderProps {
    stations: Vec<StationDto>,
}

/// The main search form + results panel.
#[component]
fn RouteFinder(props: RouteFinderProps) -> Element {
    let stations = props.stations;

    let mut source = use_storage::<LocalStorage, String>("harail_source".to_string(), String::new);
    let mut destination =
        use_storage::<LocalStorage, String>("harail_destination".to_string(), String::new);
    // Closures are only evaluated once (first mount); the current time at
    // mount becomes the live default which the user can then freely edit.
    let mut date = use_signal(|| default_date_time().0);
    let mut start_time = use_signal(|| default_date_time().1);
    let mut end_time = use_signal(|| default_date_time().2);
    let mut routes: Signal<Vec<RouteDto>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut searching = use_signal(|| false);

    // Clone for use in the results section (event handler moves other copies).
    let stations_display = stations.clone();

    let handle_swap = move |_| {
        let s = source();
        let d = destination();
        source.set(d);
        destination.set(s);
    };

    let handle_search = move |_| {
        let src = source();
        let dst = destination();
        let dt = date();
        let st = start_time();
        let et = end_time();

        if src.is_empty() || dst.is_empty() || dt.is_empty() || st.is_empty() || et.is_empty() {
            error.set(Some("Please fill in all fields.".into()));
            return;
        }

        let Ok(src_id) = src.parse::<u64>() else {
            error.set(Some("Invalid source station.".into()));
            return;
        };
        let Ok(dst_id) = dst.parse::<u64>() else {
            error.set(Some("Invalid destination station.".into()));
            return;
        };

        // Build ISO 8601 UTC timestamps from the date + time inputs.
        let start_iso = format!("{dt}T{st}:00Z");
        let end_iso = format!("{dt}T{et}:00Z");

        error.set(None);
        routes.set(vec![]);
        searching.set(true);

        spawn(async move {
            match find_routes(src_id, start_iso, dst_id, end_iso).await {
                Ok(r) => {
                    routes.set(r);
                    searching.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("{e}")));
                    searching.set(false);
                }
            }
        });
    };

    rsx! {
        h2 { "Route Finder" }

        // ── Row 1: Stations ─────────────────────────────────────────────
        div { class: "station-fields",
            div { class: "form-field",
                label { r#for: "source", "Source station" }
                select { id: "source", onchange: move |e| source.set(e.value()),
                    option { value: "", "Select source\u{2026}" }
                    for station in &stations {
                        option {
                            key: "{station.id}",
                            value: "{station.id}",
                            selected: station.id.to_string() == source(),
                            "{station.name}"
                        }
                    }
                }
            }
            button {
                class: "btn-swap",
                r#type: "button",
                title: "Swap stations",
                onclick: handle_swap,
                "\u{21C4}"
            }
            div { class: "form-field",
                label { r#for: "dest", "Destination station" }
                select { id: "dest", onchange: move |e| destination.set(e.value()),
                    option { value: "", "Select destination\u{2026}" }
                    for station in &stations {
                        option {
                            key: "{station.id}",
                            value: "{station.id}",
                            selected: station.id.to_string() == destination(),
                            "{station.name}"
                        }
                    }
                }
            }
        }

        // ── Row 2: Date & time ───────────────────────────────────────────
        div { class: "form-fields",
            div { class: "form-field",
                label { r#for: "date", "Date" }
                input {
                    id: "date",
                    r#type: "date",
                    value: date(),
                    oninput: move |e| date.set(e.value()),
                }
            }
            div { class: "form-field",
                label { r#for: "start-time", "Start time" }
                input {
                    id: "start-time",
                    r#type: "time",
                    value: start_time(),
                    oninput: move |e| start_time.set(e.value()),
                }
            }
            div { class: "form-field",
                label { r#for: "end-time", "End time" }
                input {
                    id: "end-time",
                    r#type: "time",
                    value: end_time(),
                    oninput: move |e| end_time.set(e.value()),
                }
            }
        }

        button {
            class: "btn-primary",
            onclick: handle_search,
            disabled: searching(),
            if searching() {
                "Searching…"
            } else {
                "Search Routes"
            }
        }

        // ── Feedback ────────────────────────────────────────────────────
        if let Some(err) = error() {
            p { class: "error-text", "{err}" }
        }

        // ── Results ─────────────────────────────────────────────────────
        if !searching() {
            if routes().is_empty() {
                p { class: "muted-text", "No routes found." }
            } else {
                h3 { class: "routes-header", "Routes:" }
                for (i, route) in routes().into_iter().enumerate() {
                    RouteCard {
                        key: "{i}",
                        route,
                        stations: stations_display.clone(),
                    }
                }
            }
        }
    }
}

/// Renders one route (may span multiple train legs).
#[component]
fn RouteCard(route: RouteDto, stations: Vec<StationDto>) -> Element {
    rsx! {
        div { class: "card-outlined",
            ul { class: "route-list",
                for (i, part) in route.parts.into_iter().enumerate() {
                    RoutePartItem { key: "{i}", part, stations: stations.clone() }
                }
            }
        }
    }
}

/// Renders one leg of a journey.
///
/// Clicking the item toggles a stops panel that lazily fetches every
/// scheduled stop for the train via [`get_train_stops`].
#[component]
fn RoutePartItem(part: RoutePartDto, stations: Vec<StationDto>) -> Element {
    let start_name = stations
        .iter()
        .find(|s| s.id == part.start_station)
        .map_or_else(|| "Unknown".to_owned(), |s| s.name.clone());
    let end_name = stations
        .iter()
        .find(|s| s.id == part.end_station)
        .map_or_else(|| "Unknown".to_owned(), |s| s.name.clone());
    let st = fmt_hhmm(&part.start_time);
    let et = fmt_hhmm(&part.end_time);

    let mut expanded = use_signal(|| false);
    let mut stops: Signal<Option<Vec<TrainStopDto>>> = use_signal(|| None);
    let mut loading = use_signal(|| false);
    let mut fetch_error: Signal<Option<String>> = use_signal(|| None);

    // Capture the train id for use inside the click handler.
    let train_id = part.train.clone();

    let handle_click = move |_| {
        let now_expanded = !expanded();
        expanded.set(now_expanded);

        // Only fetch once, and only when opening the panel.
        if now_expanded && stops().is_none() && !loading() {
            let tid = train_id.clone();
            loading.set(true);
            fetch_error.set(None);
            spawn(async move {
                match get_train_stops(tid).await {
                    Ok(s) => {
                        stops.set(Some(s));
                        loading.set(false);
                    }
                    Err(e) => {
                        fetch_error.set(Some(format!("{e}")));
                        loading.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        li { class: "route-part", onclick: handle_click,
            div { class: "route-part-header",
                span { class: "route-part-label", "{start_name} \u{2190} {end_name}  ({st}\u{2013}{et})" }
                span { class: "route-part-meta",
                    span { class: if expanded() { "chevron expanded" } else { "chevron" }, "\u{25bc}" }
                }
            }
            if expanded() {
                div { class: "stops-panel",
                    if loading() {
                        p { class: "loading-text", "Loading stops\u{2026}" }
                    } else if let Some(err) = fetch_error() {
                        p { class: "error-text", "Error: {err}" }
                    } else if let Some(stop_list) = stops() {
                        ul { class: "stops-list",
                            for stop in stop_list {
                                li { class: if stop.station_id == part.start_station || stop.station_id == part.end_station { "stop-item stop-item--endpoint" } else { "stop-item" },
                                    span { class: "stop-name", "{stop.station_name}" }
                                    span { class: "stop-time", "{stop.arrival_offset}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use dioxus_server::DioxusRouterExt;
    use tokio::sync::RwLock;

    use crate::db::{CACHE_FILE_NAME, load_or_download, refresh_task};

    let cache_dir = db::default_cache_dir();
    let cache_path = cache_dir.join(CACHE_FILE_NAME);

    println!("GTFS cache path: {}", cache_path.display());
    let initial_data = load_or_download(&cache_path)
        .await
        .expect("Failed to load GTFS data on startup");

    let shared: SharedData = Arc::new(RwLock::new(initial_data));
    RAILROAD_DATA
        .set(Arc::clone(&shared))
        .unwrap_or_else(|_| panic!("RAILROAD_DATA already initialised"));

    tokio::spawn(refresh_task(Arc::clone(&shared), cache_path.clone()));

    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let router = axum::Router::new()
        .serve_dioxus_application(dioxus::server::ServeConfig::new(), App)
        .into_make_service();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[cfg(not(feature = "server"))]
fn main() {
    // On native targets (desktop / Android / iOS) this sets the directory used
    // by LocalStorage to <OS data dir>/harail-app.  On WASM the macro is a
    // no-op because the browser's localStorage is used instead.
    dioxus_sdk::storage::set_dir!();

    dioxus::fullstack::set_server_url(cfg_select! {
        // In a standard Android emulator (AVD / Android Studio) the host machine
        // is reachable at 10.0.2.2. Use port 8080, the Dioxus fullstack default.
        feature = "android-dev" => "http://10.0.2.2:8080",
        _  => "https://harail.deut.sh",
    });

    dioxus::launch(App);
}
