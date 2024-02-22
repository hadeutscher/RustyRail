# HaRail

[![Build Status](https://github.com/hadeutscher/RustyRail/workflows/CI/badge.svg)](https://github.com/hadeutscher/RustyRail/actions)

Public transport fastest-route finder for Israel Railways (Rust rewrite)

## Building

`cargo build --release`

## Running from CLI

```
./harail_cli ~/harail.db parse-gtfs ~/israel-public-transportation/
./harail_cli ~/harail.db list-stations
```

If not present, obtain Israel's public transportation database from https://gtfs.mot.gov.il/gtfsfiles/israel-public-transportation.zip

Refer to `./harail_cli -h` for more options.

## Running a server

```
./harail_server ~/harail.db
```

## License

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
