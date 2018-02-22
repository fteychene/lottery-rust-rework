use std::ops::Range;
use std::clone::Clone;
use std::fmt::Debug;
use reqwest;

use rocket::http::Status;
use frunk::monoid::*;

error_chain!{
    foreign_links {
        IoError(::reqwest::Error);
    }

    errors {
        EventbriteAggregationError
    }
}

pub fn http_status_for_error(error_kind: &ErrorKind) -> Status {
    match error_kind {
        _ => Status::InternalServerError
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Pagination {
    object_count: u8,
    page_count: u8,
    page_size: u8,
    page_number: u8
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    first_name: String,
    last_name: String
}

#[derive(Deserialize, Debug, Clone)]
struct Attende {
    profile: Profile
}

#[derive(Deserialize, Debug, Clone)]
struct AttendeesResponse {
    attendees: Vec<Attende>,
    pagination: Pagination
}

fn sequence<R>(seq : Vec<Result<R>>) -> Result<Vec<R>> 
    where R: Clone + Debug {
    let result = seq.iter().fold(Ok(vec![]), |result, current| match current {
            &Ok(ref value) => result.map(|vec| { let mut x = vec.clone(); x.push(value.clone()); x }),
            &Err(ref e) => Err(ErrorKind::EventbriteAggregationError.into())
        }
    );
    result
}

fn attendees_url(event_id: &str, token: &str, page_id: u8) -> String {
    format!("https://www.eventbriteapi.com/v3/events/{event_id}/attendees/?token={token}&page={page}", event_id=event_id, token= token, page=page_id)
}

fn load_attendees(event_id: &str, token: &str, page: u8) -> Result<AttendeesResponse> {
    reqwest::get(&attendees_url(event_id, token, page))?.json().chain_err(|| "Error loading attendees on Eventbrite")
}

pub fn get_attendees(event_id: &str, token: &str) -> Result<Vec<Profile>> {
    load_attendees(event_id, token, 0)
        .and_then(|result: AttendeesResponse| {
            let range = Range{start: result.pagination.page_number, end: result.pagination.page_count};
            sequence(range.fold(vec![Ok(result)], |mut result, page|{result.push(load_attendees(event_id, token, page+1)); result}))
        })
        .map(|results: Vec<AttendeesResponse>| results.iter().map(|response| response.attendees.clone()).collect())
        .map(|results: Vec<Vec<Attende>>| combine_all(&results))
        .map(|attendees: Vec<Attende>| attendees.into_iter().map(|attendee| attendee.profile).collect())
        .chain_err(|| "Error while calling EventBrite")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence() {
        let actual = sequence( vec![Ok(1), Ok(2), Ok(3)]);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), vec![1, 2, 3]);

        let actual = sequence(vec![Ok(1), Ok(2), Err(ErrorKind::EventbriteAggregationError.into())]);
        assert!(actual.is_err());

        let actual = sequence(vec![Ok(1), Err(ErrorKind::EventbriteAggregationError.into()), Ok(3)]);
        assert!(actual.is_err());

        let actual = sequence( vec![Err(ErrorKind::EventbriteAggregationError.into()), Ok(2), Ok(3)]);
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_attendees() {
        println!("{:?}", get_attendees("39773952964", "token"));
    }
}