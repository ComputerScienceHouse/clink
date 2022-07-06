use clap::ArgMatches;

use crate::api::{APIError, API};

pub fn drop(matches: &ArgMatches, api: &mut API) -> Result<(), APIError> {
  // We can unwrap these because they're required arguments:
  let machine = matches.value_of("machine").unwrap();
  let slot = matches.value_of("slot").unwrap();

  let credits = api.drop(
    machine.to_string(),
    slot.parse().map_err(|_| APIError::BadFormat)?,
  )?;
  println!("Item dropped! Your new balance is {}", credits);
  Ok(())
}
