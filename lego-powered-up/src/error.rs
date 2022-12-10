// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use btleplug::api::ParseBDAddrError;
use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bluetooth error")]
    BluetoothError(#[from] btleplug::Error),
    #[error("NoneError: {0}")]
    NoneError(String),
    #[error("Parse error")]
    ParseErrorBLE(#[from] ParseBDAddrError),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Not implmented: {0}")]
    NotImplementedError(String),
    #[error("Hub error: {0}")]
    HubError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait OptionContext<T> {
    fn context<D: Display>(self, ctx: D) -> Result<T>;
}

impl<T> OptionContext<T> for Option<T> {
    fn context<D: Display>(self, ctx: D) -> Result<T> {
        self.ok_or_else(|| Error::NoneError(ctx.to_string()))
    }
}

impl<T> OptionContext<T> for Result<T> {
    fn context<D: Display>(self, _ctx: D) -> Result<T> {
        self.map_err(Error::from)
    }
}
