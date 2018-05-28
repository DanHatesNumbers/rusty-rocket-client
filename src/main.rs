#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::io::{self, Write};
use std::str::FromStr;
use futures::{Future, Stream};
use hyper::{Client, Uri};
use hyper::error::{Error, UriError};
use tokio_core::reactor::Core;

#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "IO Error: {}", _0)]
    IoError(std::io::Error),
    #[fail(display = "Error processing hyper request: {}", _0)]
    HyperError(Error),
    #[fail(display = "Error parsing URI: {}", _0)]
    UriParseError(UriError)
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<Error> for AppError {
    fn from(error: Error) -> Self {
        AppError::HyperError(error)
    }
}

impl From<UriError> for AppError {
    fn from(error: UriError) -> Self {
        AppError::UriParseError(error)
    }
}

fn main() -> Result<(), AppError> {
    let mut core = Core::new()?;
    let client = Client::new(&core.handle());

    let uri = Uri::from_str("http://192.168.99.100:8080/people")?;

    let work = client.get(uri).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });

    core.run(work)
        .map_err(|error| error.into())
}