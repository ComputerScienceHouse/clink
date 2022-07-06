use clap::ArgMatches;

use crate::api::{APIError, API};

pub fn list(matches: &ArgMatches, api: &mut API) -> Result<(), APIError> {
  let drinks = api.get_status_for_machine(matches.value_of("machine"))?;

  for machine in drinks.machines {
    let subject_line = format!("{} ({})", machine.display_name, machine.name);
    println!("{}", &subject_line);
    println!("{}", "=".repeat(subject_line.len()));
    for slot in machine.slots {
      let item = slot.item;
      print!("{}. {} ({} Credits)", slot.number, item.name, item.price);
      if slot.empty {
        print!(" [EMPTY]");
      }
      println!();
    }
  }
  Ok(())
}
