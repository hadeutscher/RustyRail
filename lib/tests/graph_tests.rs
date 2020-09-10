/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod test_data;
use chrono::{NaiveDateTime, NaiveTime};
use harail::RoutePart;
use harail::{HaDuration, RailroadData, StopSchedule, Train};
use test_data::test_date;

#[test]
fn shortest_path() {
    // Basic shortest-path test, expected result is to ride train 2 from 100 to 400 and then go back to 300 using train 3
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(10, 30, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "3",
        vec![
            StopSchedule::new(400, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(10, 40, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(400, trains[0].end().station().id());
    assert_eq!("3", trains[1].train().id());
    assert_eq!(400, trains[1].start().station().id());
    assert_eq!(300, trains[1].end().station().id());

    let route = harail::get_latest_good_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(400, trains[0].end().station().id());
    assert_eq!("3", trains[1].train().id());
    assert_eq!(400, trains[1].start().station().id());
    assert_eq!(300, trains[1].end().station().id());
}

#[test]
fn minimize_switches() {
    // Tests basic train switch cost logic. Expected result is to ride train 1 all the way to station 600,
    // even though some of the route appears to be faster if we switch train.
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
            StopSchedule::new(500, HaDuration::from_hms(12, 00, 00), None),
            StopSchedule::new(600, HaDuration::from_hms(12, 30, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(200, HaDuration::from_hms(10, 31, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(10, 32, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "3",
        vec![
            StopSchedule::new(500, HaDuration::from_hms(12, 01, 00), None),
            StopSchedule::new(600, HaDuration::from_hms(12, 30, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(13, 00, 00)),
        data.station(600),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(600, trains[0].end().station().id());

    let route = harail::get_latest_good_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(13, 00, 00)),
        data.station(600),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(600, trains[0].end().station().id());
}

#[test]
fn minimize_switches2() {
    // Test train switch minimization in a more complicated case. Expected result is to only use train 1.
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 01, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(400, HaDuration::from_hms(11, 30, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(10, 30, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(400),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(400, trains[0].end().station().id());

    let route = harail::get_latest_good_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(400),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(400, trains[0].end().station().id());
}

#[test]
fn wait_on_train() {
    // Test trains with WAIT_ON_TRAIN
    // expected result is to use train 1 only in train switching mode, train 2 then 1 in delayed leaving
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(
                200,
                HaDuration::from_hms(10, 20, 00),
                Some(HaDuration::from_hms(10, 30, 00)),
            ),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 10, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 20, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_best_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(300, trains[0].end().station().id());

    let route = harail::get_latest_good_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(200, trains[0].end().station().id());
    assert_eq!("1", trains[1].train().id());
    assert_eq!(200, trains[1].start().station().id());
    assert_eq!(300, trains[1].end().station().id());
}

#[test]
fn wait_on_train_alt_route() {
    // Test trains with WAIT_ON_TRAIN and alt-route finding
    // expected result is to use train 2 and then switch to 3
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(
                200,
                HaDuration::from_hms(10, 20, 00),
                Some(HaDuration::from_hms(10, 30, 00)),
            ),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 10, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 20, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "3",
        vec![
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(10, 40, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let route = harail::get_latest_good_single_route(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    )
    .unwrap();
    let trains: Vec<&RoutePart> = route.parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(200, trains[0].end().station().id());
    assert_eq!("3", trains[1].train().id());
    assert_eq!(200, trains[1].start().station().id());
    assert_eq!(300, trains[1].end().station().id());
}

#[test]
fn multiple_routes() {
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 30, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(11, 00, 00), None),
            StopSchedule::new(300, HaDuration::from_hms(11, 30, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let routes = harail::get_multiple_routes(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    );
    assert_eq!(2, routes.len());
    let trains: Vec<&RoutePart> = routes[0].parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(300, trains[0].end().station().id());
    let trains: Vec<&RoutePart> = routes[1].parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(300, trains[0].end().station().id());
}

#[test]
fn wait_on_train_multiple_routes() {
    let mut trains = Vec::new();
    trains.push(Train::from_stops_date(
        "1",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 00, 00), None),
            StopSchedule::new(
                200,
                HaDuration::from_hms(10, 20, 00),
                Some(HaDuration::from_hms(10, 30, 00)),
            ),
            StopSchedule::new(300, HaDuration::from_hms(11, 00, 00), None),
        ],
        test_date(),
    ));
    trains.push(Train::from_stops_date(
        "2",
        vec![
            StopSchedule::new(100, HaDuration::from_hms(10, 10, 00), None),
            StopSchedule::new(200, HaDuration::from_hms(10, 20, 00), None),
        ],
        test_date(),
    ));
    let data = RailroadData::from_stations_trains(test_data::stations(), trains);
    let routes = harail::get_multiple_routes(
        &data,
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(10, 00, 00)),
        data.station(100),
        NaiveDateTime::new(test_date(), NaiveTime::from_hms(12, 00, 00)),
        data.station(300),
    );
    assert_eq!(2, routes.len());
    let trains: Vec<&RoutePart> = routes[0].parts().collect();
    assert_eq!(1, trains.len());
    assert_eq!("1", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(300, trains[0].end().station().id());
    let trains: Vec<&RoutePart> = routes[1].parts().collect();
    assert_eq!(2, trains.len());
    assert_eq!("2", trains[0].train().id());
    assert_eq!(100, trains[0].start().station().id());
    assert_eq!(200, trains[0].end().station().id());
    assert_eq!("1", trains[1].train().id());
    assert_eq!(200, trains[1].start().station().id());
    assert_eq!(300, trains[1].end().station().id());
}
