# HaRail

Public transport fastest-route finder for Israel Railways (Rust rewrite)

## Building

`cargo build --release`

## Running

```
./harail ~/harail.db parse-gtfs ~/israel-public-transportation/
./harail ~/harail.db list-stations
```

If not present, obtain Israel's public transportation database from ftp://gtfs.mot.gov.il/israel-public-transportation.zip

Refer to `./harail -h` for more options.

## License

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
