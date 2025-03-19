# HaRail

[![Build Status](https://github.com/hadeutscher/RustyRail/workflows/CI/badge.svg)](https://github.com/hadeutscher/RustyRail/actions)

Public transport fastest-route finder for Israel Railways (Rust rewrite)

## Building

`cargo build --release`

## Running from CLI

```
./harail ~/harail.db parse-gtfs ~/israel-public-transportation/
./harail ~/harail.db list-stations
```

If not present, obtain Israel's public transportation database from https://gtfs.mot.gov.il/gtfsfiles/israel-public-transportation.zip

Refer to `./harail -h` for more options.

## Running a server

### With docker

```
docker run -p 8080:8080 -v $(pwd)/harail.db:/harail.db ghcr.io/hadeutscher/harail /harail.db
```

### With docker compose

```
docker compose up -d
```

Then copy the db into the `db-data` volume.

## License

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
