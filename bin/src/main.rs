#![recursion_limit = "1024"]

#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate frunk;
extern crate lottery_eventbrite;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;

use rocket_contrib::Json;

use frunk::monoid::*;

error_chain!{}

#[derive(Serialize)]
struct Health {
    status: bool,
    application: String
}

#[get("/health")]
fn health() -> Result<Json<Health>> {
    Ok(Json(Health{status: true, application: String::from("lottery-jug")}))
}

fn main() {
    rocket::ignite().mount("/", combine_all(&vec!(routes![health], lottery_eventbrite::handlers()))).launch();
}
