#![recursion_limit = "1024"]
#![feature(match_default_bindings)]
#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate frunk;
#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod events;
pub mod attendees;
