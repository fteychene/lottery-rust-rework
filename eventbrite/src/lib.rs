#![recursion_limit = "1024"]
#![feature(match_default_bindings)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate frunk;

mod events;
mod attendees;

use rocket_contrib::Json;
use rocket::{Route, Request, Response};
use rocket::response::Responder;
use rocket::http::{ContentType, Status};
use std::io::Cursor;
use events::{Event};
use attendees::{Profile};

error_chain!{
    links {
        Events(events::Error, events::ErrorKind);
        Attendees(attendees::Error, attendees::ErrorKind);
    }
}

#[derive(Serialize)]
struct LotteryHttpError {
    err: String
}

fn response_error<E>(error: &E, status: Status) -> ::std::result::Result<Response<'static>, Status> 
where E: std::error::Error {
    Response::build()
        .header(ContentType::JSON)
        .status(status)
        .sized_body(Cursor::new(serde_json::to_string(&LotteryHttpError{err: String::from(error.description())}).unwrap()))
        .ok()
}

impl Responder<'static> for Error {

    fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'static>, Status> {
        use error_chain::ChainedError;
        eprintln!("An error occured during request : {}", self.display_chain());
        
        match self.kind() {
            &ErrorKind::Events(ref error_kind) => response_error(&self, events::http_status_for_error(error_kind)),
            &ErrorKind::Attendees(ref error_kind) => response_error(&self, attendees::http_status_for_error(error_kind)),
            _ => response_error(&self, Status::InternalServerError)
        }
    }
}

#[get("/event")]
fn event() -> Result<Json<Event>> {
    let result = events::get_current_event("1464915124", "token")?;
    Ok(Json(result))
}

#[get("/attendees")]
fn attendees() -> Result<Json<Vec<Profile>>> {
    //let current_event = events::get_current_event("1464915124", "token")?;
    let attendees = attendees::get_attendees("32403387404", "token")?;
    Ok(Json(attendees))
}

pub fn handlers() -> Vec<Route> {
    routes![event, attendees]
}