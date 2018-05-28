#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::io::{self, Write};
use std::str::{self, FromStr};
use futures::{Future, future, Stream};
use hyper::{Client, Method, Request, Uri};
use hyper::error::{Error, UriError};
use hyper::header::{ContentLength, ContentType};
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

    let dan = r#"{"id": 0, "first_name": "Dan", "last_name": "Murphy", "age": 26, "profession": "Quality Engineer", "salary": 45000}"#;
    let graham = r#"{"id": 0, "first_name": "Graham", "last_name": "Sutherland", "age": 30, "profession": "Hacker", "salary": 55000}"#;

    let mut dan_req = Request::new(Method::Post, uri.clone());
    dan_req.headers_mut().set(ContentType::json());
    dan_req.headers_mut().set(ContentLength(dan.len() as u64));
    dan_req.set_body(dan);
    let dan_post = client.request(dan_req);
    
    let mut graham_req = Request::new(Method::Post, uri.clone());
    graham_req.headers_mut().set(ContentType::json());
    graham_req.headers_mut().set(ContentLength(graham.len() as u64));
    graham_req.set_body(graham);

    let graham_post = client.request(graham_req);

    let work = future::join_all(vec![dan_post, graham_post])
        .and_then(|results| {
            future::ok(results.into_iter()
                .map(|result| {
                    println!("Status: {}", result.status());

                    result.body().concat2()
                })
                .collect::<Vec<_>>()
            )
        });

    let results: Vec<futures::stream::Concat2<hyper::Body>> = core.run(work).unwrap();

    let parsed_results = results.into_iter()
        .map(|result| {
            str::from_utf8(&result.wait().unwrap()).unwrap().to_string()
        })
        .collect::<Vec<_>>();

    for body in parsed_results {
        println!("{}", body);
    }

    Ok(())
}