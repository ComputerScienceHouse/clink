use clap::ArgMatches;

use crate::api::{APIError, API};

pub fn token(_matches: &ArgMatches, api: &mut API) -> Result<(), APIError> {
  println!("{}", api.get_token()?);

  Ok(())
}
