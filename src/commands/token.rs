use clap::ArgMatches;

use crate::api::API;

pub fn token(_matches: &ArgMatches, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  println!("{}", api.get_token()?);

  Ok(())
}
