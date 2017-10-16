extern crate toml;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod config;
mod slack_bot;

fn main() {
    slack_bot::init();
}