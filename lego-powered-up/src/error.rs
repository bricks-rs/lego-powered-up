use btleplug::api::ParseBDAddrError;
use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bluetooth error")]
    BluetoothError(#[from] btleplug::Error),
    #[error("NoneError: {0}")]
    NoneError(String),
    #[error("Channel error")]
    ChannelError(CrossbeamError),
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

impl<E> From<E> for Error
where
    E: Into<CrossbeamError>,
{
    fn from(e: E) -> Error {
        Error::ChannelError(e.into())
    }
}

#[derive(Debug)]
pub struct CrossbeamError(String);

impl From<crossbeam_channel::RecvError> for CrossbeamError {
    fn from(e: crossbeam_channel::RecvError) -> Self {
        Self(e.to_string())
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for CrossbeamError {
    fn from(e: crossbeam_channel::SendError<T>) -> Self {
        Self(e.to_string())
    }
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
