use isahc::{Request, HttpClient, ReadResponseExt};
use isahc::error::Error;
use clap::ArgMatches;
use std::any::Any;

use serde_json::{Map, Value};

use crate::api::API;

pub fn list(matches: &ArgMatches<'_>, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  let token = api.get_token()?;

  let client = HttpClient::new()?;
  let request = Request::get("https://drink.csh.rit.edu/drinks")
    .header("Authorization", token)
    .body(())?;

  let drinks: Value = client.send(request)?.json()?;
  let drinks: &Map<String, Value> = match drinks.as_object() {
    Some(drinks) => drinks,
    None => panic!("Fuck")
  };
  let machines: &Vec<Value> = match drinks["machines"].as_array() {
    Some(machines) => machines,
    None => panic!("Fuck")
  };
  for machine in machines {
    let machine: &Map<String, Value> = match machine.as_object() {
      Some(machine) => machine,
      None => panic!("Fuck!")
    };
    println!("Heyy {}", machine["display_name"].as_str().unwrap().to_string());
  }
  return Ok(());
}
