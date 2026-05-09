/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{error::Error, fs::File, io::prelude::*, path::Path};
use zip::{ZipArchive, read::ZipFile};

pub trait FileOpener {
    type Read<'a>: Read
    where
        Self: 'a;

    fn open(&mut self, name: &str) -> Result<Self::Read<'_>, Box<dyn Error>>;
}

pub struct PathFileOpener<'p> {
    path: &'p Path,
}

impl<'p> PathFileOpener<'p> {
    pub fn new(path: &'p Path) -> Self {
        PathFileOpener { path }
    }
}

impl FileOpener for PathFileOpener<'_> {
    type Read<'a>
        = File
    where
        Self: 'a;

    fn open(&mut self, name: &str) -> Result<Self::Read<'_>, Box<dyn Error>> {
        Ok(File::open(self.path.join(name))?)
    }
}

pub struct ZipFileOpener<R: Read + Seek> {
    zip: ZipArchive<R>,
}

impl<R: Read + Seek> ZipFileOpener<R> {
    pub fn new(zip: ZipArchive<R>) -> Self {
        ZipFileOpener { zip }
    }
}

impl<R: Read + Seek> FileOpener for ZipFileOpener<R> {
    type Read<'a>
        = ZipFile<'a, R>
    where
        R: 'a;

    fn open(&mut self, name: &str) -> Result<Self::Read<'_>, Box<dyn Error>> {
        Ok(self.zip.by_name(name)?)
    }
}
