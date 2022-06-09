use clap::ArgMatches;

use crate::api::API;

pub fn drop(matches: &ArgMatches, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  // We can unwrap these because they're required arguments:
  let machine = matches.value_of("machine").unwrap();
  let slot = matches.value_of("slot").unwrap();

  let credits = api.drop(machine.to_string(), slot.parse()?)?;
  println!("Item dropped! Your new balance is {}", credits);
  Ok(())
}
