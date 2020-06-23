use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

extern crate harail;
use harail::gtfs::{RailroadData, Station, Stop, Train};
use harail::RoutePart;

fn stations() -> Vec<Station> {
    vec![
        Station::new(100, "stat_a"),
        Station::new(200, "stat_b"),
        Station::new(300, "stat_c"),
        Station::new(400, "stat_d"),
        Station::new(500, "stat_e"),
        Station::new(600, "stat_f"),
    ]
}

fn time(h: u32, m: u32, s: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd(2000, 01, 01),
        NaiveTime::from_hms(h, m, s),
    )
}

#[test]
fn shortest_path() {
    // Basic shortest-path test, expected result is to ride train 2 from 100 to 400 and then go back to 300 using train 3
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(200, time(10, 30, 00), None),
            Stop::new(300, time(11, 00, 00), None),
            Stop::new(400, time(11, 30, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "2",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(400, time(10, 30, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "3",
        vec![
            Stop::new(400, time(10, 30, 00), None),
            Stop::new(300, time(10, 40, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        time(10, 00, 00),
        data.station(100),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station());
    assert_eq!(400, trains[0].end().station());
    assert_eq!("3", trains[1].train().id());
    assert_eq!(400, trains[1].start().station());
    assert_eq!(300, trains[1].end().station());
}

#[test]
fn minimize_switches() {
    // Tests basic train switch cost logic. Expected result is to ride train 1 all the way to station 600,
    // even though some of the route appears to be faster if we switch train.
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(200, time(10, 30, 00), None),
            Stop::new(300, time(11, 00, 00), None),
            Stop::new(400, time(11, 30, 00), None),
            Stop::new(500, time(12, 00, 00), None),
            Stop::new(600, time(12, 30, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "2",
        vec![
            Stop::new(200, time(10, 31, 00), None),
            Stop::new(400, time(10, 32, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "3",
        vec![
            Stop::new(500, time(12, 01, 00), None),
            Stop::new(600, time(12, 30, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        time(10, 00, 00),
        data.station(100),
        data.station(600),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station());
    assert_eq!(600, trains[0].end().station());
}

#[test]
fn minimize_switches2() {
    // Test train switch minimization in a more complicated case. Expected result is to only use train 1.
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 01, 00), None),
            Stop::new(200, time(10, 30, 00), None),
            Stop::new(300, time(11, 00, 00), None),
            Stop::new(400, time(11, 30, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "2",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(300, time(10, 30, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        time(10, 00, 00),
        data.station(100),
        data.station(400),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station());
    assert_eq!(400, trains[0].end().station());
}

#[test]
fn wait_on_train() {
    // Test trains with WAIT_ON_TRAIN
    // expected result is to use train 1 only in train switching mode, train 2 then 1 in delayed leaving
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(200, time(10, 20, 00), Some(time(10, 30, 00))),
            Stop::new(300, time(11, 00, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "2",
        vec![
            Stop::new(100, time(10, 10, 00), None),
            Stop::new(200, time(10, 20, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        time(10, 00, 00),
        data.station(100),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station());
    assert_eq!(300, trains[0].end().station());

    // TODO: Add rest of test after alt-route finding is implemented
}


#[test]
fn wait_on_train_alt_route() {
    // Test trains with WAIT_ON_TRAIN and alt-route finding
    // expected result is to use train 2 and then switch to 3
    let mut trains = Vec::new();
    trains.push(Train::from_stops(
        "1",
        vec![
            Stop::new(100, time(10, 00, 00), None),
            Stop::new(200, time(10, 20, 00), Some(time(10, 30, 00))),
            Stop::new(300, time(11, 00, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "2",
        vec![
            Stop::new(100, time(10, 10, 00), None),
            Stop::new(200, time(10, 20, 00), None),
        ],
    ));
    trains.push(Train::from_stops(
        "3",
        vec![
            Stop::new(200, time(10, 30, 00), None),
            Stop::new(300, time(10, 40, 00), None),
        ],
    ));
    let data = RailroadData::from_stations_trains(stations(), trains);
    
    // TODO: Add rest of test after alt-route finding is implemented
}