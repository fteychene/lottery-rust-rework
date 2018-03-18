#![recursion_limit = "1024"]
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate frunk;
extern crate lottery_eventbrite;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate rand;

mod eventbrite;

use rocket_contrib::Json;
use rocket::http::{ContentType, Status};
use std::io::Cursor;
use rocket::{Response, State};
use std::env;

use std::sync::RwLock;
use frunk::monoid::combine_all;
use lottery_eventbrite::attendees::Profile;
use std::thread;
use rand::{seq, thread_rng};

error_chain!{
    errors {
        CacheError
        NoEventAvailable
    }
}

#[derive(Serialize)]
struct LotteryHttpError {
    err: String,
}

fn response_error<E>(error: &E, status: Status) -> ::std::result::Result<Response<'static>, Status>
where
    E: std::error::Error,
{
    Response::build()
        .header(ContentType::JSON)
        .status(status)
        .sized_body(Cursor::new(
            serde_json::to_string(&LotteryHttpError {
                err: String::from(error.description()),
            }).unwrap(),
        ))
        .ok()
}

#[derive(Serialize)]
struct Health {
    status: bool,
    application: String,
}

#[get("/health")]
fn health() -> Result<Json<Health>> {
    Ok(Json(Health {
        status: true,
        application: String::from("lottery-jug"),
    }))
}

#[derive(FromForm)]
struct WinnerParams {
    nb: u8,
}

#[get("/winners?<params>")]
fn winners(
    params: WinnerParams,
    state: State<&RwLock<Option<Vec<Profile>>>>,
) -> Result<Json<Vec<Profile>>> {
    let mut rng = thread_rng();
    state
        .read()
        .map_err(|e| {
            eprintln!("Error accessing cache : {}", e);
            ErrorKind::CacheError.into()
        })
        .and_then(|guard| guard.clone().ok_or(ErrorKind::NoEventAvailable.into()))
        .map(|attendees| match params.nb {
            0 => Json(attendees.clone()),
            number => Json(seq::sample_iter(&mut rng, attendees, number as usize).unwrap())
        })
}

lazy_static! {
    static ref ATTENDEES: RwLock<Option<Vec<Profile>>> = RwLock::new(None);
}

fn main() {
    let organizer = env::var("ORGANIZER_TOKEN").unwrap();
    let token = env::var("EVENTBRITE_TOKEN").unwrap();

    thread::spawn(move || {
        eventbrite::cache_loop(&ATTENDEES, &organizer, &token, 3600)
    });
    let attendees_ref: &RwLock<Option<Vec<Profile>>> = &ATTENDEES;
    rocket::ignite()
        .manage(attendees_ref)
        .mount(
            "/",
            combine_all(&vec![routes![health, winners]]),
        )
        .launch();
}
