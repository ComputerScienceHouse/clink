use clap::ArgMatches;

use crate::api::API;

pub fn credits(_matches: &ArgMatches, api: &mut API) -> Result<(), Box<dyn std::error::Error>> {
  let credits = api.get_credits()?;
  println!("{} credits", credits);

  Ok(())
}
