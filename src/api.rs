use std::fmt;
use url::Url;
use isahc::{Request, auth::Authentication, prelude::*, HttpClient};
use serde_json::{Map, Value};

use crate::ui::inventory;

pub struct API {
  token: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
  Unauthorized,
  BadFormat
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return f.write_str(match self {
      Unauthorized => "Unauthorized (Did your Kerberos ticket expire?: `kinit`)",
      BadFormat => "BadFormat (The server sent data we didn't understand)"
    });
  }
}

impl API {
  pub fn new() -> API {
    return API {
      token: None
    };
  }
  pub fn get_token(self: &mut API) -> Result<String, Box<dyn std::error::Error>> {
    return match &self.token {
      Some(token) => Ok(token.to_string()),
      None => {
        let response = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?client_id=clidrink&redirect_uri=drink%3A%2F%2Fcallback&response_type=token%20id_token&scope=openid%20profile%20drink_balance&state=&nonce=")
          .authentication(Authentication::negotiate())
          .body(())?.send()?;
        let location = match response.headers().get("Location") {
          Some(location) => location,
          None => return Err(Box::new(APIError::Unauthorized)),
        };
        let url = Url::parse(&location.to_str()?.replace("#", "?"))?;

        for (key, value) in url.query_pairs() {
          if key == "access_token" {
            let value = "Bearer ".to_owned() + &value.to_string();
            self.token = Some(value.to_string());
            return Ok(value.to_string());
          }
        }
        return Err(Box::new(APIError::BadFormat));
      }
    }
  }
}

pub fn get_machines(api: &mut API) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let token = api.get_token()?;

  let client = HttpClient::new()?;
  let request = Request::get("https://drink.csh.rit.edu/drinks")
    .header("Authorization", token)
    .body(())?;

  let mut display_names = Vec::new();

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
//    eprintln!("Heyy {}", machine["display_name"].as_str().unwrap().to_string());
    display_names.push(machine["display_name"].as_str().unwrap().to_string());
  }
  return Ok(display_names);
}

//pub fn get_inventory(api: &mut API, machine_index: i32) -> Result<Vec<inventory::Item>, Box<dyn std::error::Error>> {
//  let token = api.get_token()?;
//
//  let client = HttpClient::new()?;
//  let request = Request::get("https://drink.csh.rit.edu/drinks")
//    .header("Authorization", token)
//    .body(())?;
//
//  let mut display_names = Vec::new();
//
//  let drinks: Value = client.send(request)?.json()?;
//  let drinks: &Map<String, Value> = match drinks.as_object() {
//    Some(drinks) => drinks,
//    None => panic!("Fuck")
//  };
//  let machines: &Vec<Value> = match drinks["machines"].as_array() {
//    Some(machines) => machines,
//    None => panic!("Fuck")
//  };
//
//  // Fucking bullshit.
//  let slots: &Vec<Value> = match machines.get(machine_index as usize).as_object()["slots"] {
//    Some(slots) => slots,
//    None => panic!("Fuck")
//  };
//
//  for slot in slots {
//    let slot: &Map<String, Value> = match slot.as_object() {
//      Some(slot) => slot,
//      None => panic!("Fuck!")
//    };
////    eprintln!("Heyy {}", machine["display_name"].as_str().unwrap().to_string());
////Item {name: "Coke".to_string(), price: 10}
//    display_names.push(Item { name: slot["item"]["name"].as_str().unwrap().to_string(), price: 69 });
//  }
//  return Ok(display_names);
//}
