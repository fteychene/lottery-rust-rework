#![recursion_limit = "1024"]

#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate frunk;
extern crate lottery_eventbrite;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use] 
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

mod eventbrite;

use rocket_contrib::Json;
use rocket::http::{ContentType, Status};
use std::io::Cursor;
use rocket::{State, Response};

use std::sync::Mutex;
use frunk::monoid::{combine_all};
use lottery_eventbrite::attendees::Profile;
use std::thread;

error_chain!{
    errors {
        CacheError
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

#[derive(Serialize)]
struct Health {
    status: bool,
    application: String
}

#[get("/health")]
fn health() -> Result<Json<Health>> {
    Ok(Json(Health{status: true, application: String::from("lottery-jug")}))
}

#[get("/winners")]
fn winners(attendees: State<&Mutex<Vec<Profile>>>) -> Result<Json<Vec<Profile>>> {
    attendees.lock()
        .map(|attendees| Json(attendees.clone()))
        .map_err(|e| {
            eprintln!("Error accessing cache : {}", e);
            ErrorKind::CacheError.into()
        })
}

lazy_static! {
    static ref ATTENDEES: Mutex<Vec<Profile>> = Mutex::new(vec![]);
}

fn main() {
    thread::spawn(|| eventbrite::cache_loop(&ATTENDEES, "id", "token", 3600));
    let attendees_ref: &Mutex<Vec<Profile>> = &ATTENDEES;
    rocket::ignite()
        .manage(attendees_ref)
        .mount("/", combine_all(&vec!(routes![health, winners], eventbrite::handlers())))
        .launch();
}
