use reqwest;
use rocket::http::Status;

error_chain!{
    errors { 
        NoEventsAvailable {
            description("No events available")
            display("No event available in EventBrite")
        }
    }

    foreign_links {
        IoError(::reqwest::Error);
    }
}

pub fn http_status_for_error(error_kind: &ErrorKind) -> Status {
    match error_kind {
        &ErrorKind::NoEventsAvailable => Status::BadRequest,
        _ => Status::InternalServerError
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: String
}

#[derive(Deserialize, Debug)]
struct EventsResponse {
    pub events: Vec<Event>
}

fn load_events(organizer: &str, token: &str) -> Result<EventsResponse> {
    reqwest::get(&format!("https://www.eventbriteapi.com/v3/events/search/?sort_by=date&organizer.id={organizer}&token={token}", organizer=organizer, token=token))?
        .json()
        .chain_err(|| "Error while calling EventBrite")
}

fn first_event(events: EventsResponse) -> Result<Event> {
    events.events.first().map(|reference| reference.clone()).ok_or(ErrorKind::NoEventsAvailable.into())
}

pub fn get_current_event(organizer: &str, token: &str) -> Result<Event> {
    load_events(organizer, token).and_then(first_event)
}

