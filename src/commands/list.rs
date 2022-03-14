use clap::ArgMatches;
use isahc::{HttpClient, ReadResponseExt, Request};

use serde_json::{Map, Value};

use crate::api::APIError;
use crate::api::API;

pub fn list(matches: &ArgMatches, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  let token = api.get_token()?;

  let client = HttpClient::new()?;
  let mut url = "https://drink.csh.rit.edu/drinks".to_string();
  if let Some(machine) = matches.value_of("machine") {
    url += "?machine=";
    url += machine;
  }
  let request = Request::get(url).header("Authorization", token).body(())?;

  let drinks: Value = client.send(request)?.json()?;
  let drinks: &Map<String, Value> = match drinks.as_object() {
    Some(drinks) => drinks,
    None => panic!("Fuck"),
  };
  let machines: &Vec<Value> = match drinks.get("machines") {
    Some(machines) => match machines.as_array() {
      Some(machines) => machines,
      None => return Err(Box::new(APIError::BadFormat)),
    },
    None => {
      eprintln!("Couldn't fetch machines! {:?}", drinks);
      return Err(Box::new(APIError::BadFormat));
    }
  };
  for machine in machines {
    let machine: &Map<String, Value> = match machine.as_object() {
      Some(machine) => machine,
      None => panic!("Fuck!"),
    };
    let display_name = match machine["display_name"].as_str() {
      Some(name) => name.to_string(),
      None => return Err(Box::new(APIError::BadFormat)),
    };
    let name = match machine["name"].as_str() {
      Some(name) => name.to_string(),
      None => return Err(Box::new(APIError::BadFormat)),
    };
    let subject_line = format!("{} ({})", display_name, name);
    println!("{}", &subject_line);
    println!("{}", "=".repeat(subject_line.len()));
    let slots: &Vec<Value> = match machine["slots"].as_array() {
      Some(slots) => slots,
      None => return Err(Box::new(APIError::BadFormat)),
    };
    for slot in slots {
      let slot: &Map<String, Value> = match slot.as_object() {
        Some(slot) => slot,
        None => return Err(Box::new(APIError::BadFormat)),
      };

      let item: &Map<String, Value> = match slot["item"].as_object() {
        Some(item) => item,
        None => return Err(Box::new(APIError::BadFormat)),
      };

      let price = item["price"].as_u64().unwrap();
      let slot_number = slot["number"].as_u64().unwrap();
      let name = item["name"].as_str().unwrap();
      print!("{}. {} ({} Credits)", slot_number, name, price);
      if slot["empty"].as_bool().unwrap() {
        print!(" [EMPTY]");
      }
      println!("");
    }
  }
  return Ok(());
}
