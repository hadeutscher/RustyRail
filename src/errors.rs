/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::io::ErrorKind;

pub fn make_error(m: &str) -> Box<dyn Error> {
    Box::new(std::io::Error::new(ErrorKind::InvalidInput, m))
}
