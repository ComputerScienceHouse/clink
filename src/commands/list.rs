use clap::ArgMatches;
use isahc::{HttpClient, ReadResponseExt, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use std::io::prelude::*;

use crate::api::APIError;
use crate::api::API;

#[derive(Debug, Deserialize)]
struct Drinks {
  machines: Vec<Machine>,
}

#[derive(Debug, Deserialize)]
struct Machine {
  name: String,
  display_name: String,
  slots: Vec<Slot>,
}

impl fmt::Display for Machine {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} ({})\n{}",
      self.display_name,
      self.name,
      "=".repeat(self.display_name.len() + self.name.len() + 3)
    )
  }
}

#[derive(Debug, Deserialize)]
struct Slot {
  item: Item,
  number: u64,
  empty: bool,
  active: bool,
  count: Option<u16>,
}

impl fmt::Display for Slot {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}. {}{}",
      self.number,
      self.item,
      match self.empty {
        true => " [EMPTY]",
        false => "",
      }
    )
  }
}

#[derive(Debug, Deserialize)]
struct Item {
  name: String,
  price: u64,
}

impl fmt::Display for Item {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} ({} Credits)", self.name, self.price)
  }
}

pub fn list(matches: &ArgMatches, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  let token = api.get_token()?;

  let mut term = term::stdout().unwrap();

  let client = HttpClient::new()?;
  let mut url = "https://drink.csh.rit.edu/drinks".to_string();
  if let Some(machine) = matches.value_of("machine") {
    url += "?machine=";
    url += machine;
  }
  let request = Request::get(url).header("Authorization", token).body(())?;

  let drinks: Drinks = client.send(request)?.json()?;
  for machine in drinks.machines {
    term.fg(term::color::CYAN).unwrap();
    writeln!(term, "{}", machine).unwrap();
    for slot in machine.slots {
      match slot.count {
        Some(0) => term.fg(term::color::RED),
        Some(_) => term.reset(),
        None => {
          if slot.empty || !slot.active {
            term.fg(term::color::RED)
          } else {
            term.reset()
          }
        }
      }
      .unwrap();
      writeln!(term, "{}", slot).unwrap();
    }
    println!("");
    term.reset().unwrap();
  }
  term.flush().unwrap();
  return Ok(());
}
