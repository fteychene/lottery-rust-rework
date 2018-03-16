
use rocket_contrib::Json;
use rocket::{Route, Request, Response};
use rocket::response::Responder;
use rocket::http::Status;
use lottery_eventbrite::events;
use lottery_eventbrite::attendees;
use super::response_error;
use std::{thread, time};
use std::sync::Mutex;

error_chain!{
    links {
        Events(events::Error, events::ErrorKind);
        Attendees(attendees::Error, attendees::ErrorKind);
    }
}


impl Responder<'static> for Error {

    fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'static>, Status> {
        use error_chain::ChainedError;
        eprintln!("An error occured during request : {}", self.display_chain());
        
        match self.kind() {
            &ErrorKind::Events(ref error_kind) => response_error(&self, http_status_for_event_error(error_kind)),
            &ErrorKind::Attendees(ref error_kind) => response_error(&self, http_status_for_attendees_error(error_kind)),
            _ => response_error(&self, Status::InternalServerError)
        }
    }
}


fn http_status_for_attendees_error(error_kind: &attendees::ErrorKind) -> Status {
    match error_kind {
        _ => Status::InternalServerError
    }
}

fn http_status_for_event_error(error_kind: &events::ErrorKind) -> Status {
    match error_kind {
        &events::ErrorKind::NoEventsAvailable => Status::BadRequest,
        _ => Status::InternalServerError
    }
}


#[get("/event")]
fn event() -> Result<Json<events::Event>> {
    let result = events::get_current_event("1464915124", "token")?;
    Ok(Json(result))
}

#[get("/attendees")]
fn attendees() -> Result<Json<Vec<attendees::Profile>>> {
    //let current_event = events::get_current_event("1464915124", "token")?;
    // let attendees = attendees::get_attendees("32403387404", "token")?;
    let attendees = load_current_attendees("1464915124", "token")?;
    Ok(Json(attendees))
}

fn load_current_attendees(organizer: &str, token: &str) -> Result<Vec<attendees::Profile>> {
    let current_event = events::get_current_event(organizer, token)?;
    attendees::get_attendees(&current_event.id, token).chain_err(|| "error loading attendees")
}

pub fn cache_loop(attendees: &Mutex<Vec<attendees::Profile>>, organizer: &str, token: &str, timeout: u64) {
    loop {
        println!("Fetch last event and attendees from eventbrite");

        match load_current_attendees(organizer, token) {
            Ok(current_attendees) => *attendees.lock().unwrap() = current_attendees,
            Err(e) => eprintln!("Error on loading last event and attendees : {}", e)
        }
        
        thread::sleep(time::Duration::from_secs(timeout));
    }
}

pub fn handlers() -> Vec<Route> {
    routes![event, attendees]
}