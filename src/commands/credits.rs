use clap::ArgMatches;

use crate::api::{APIError, API};

pub fn credits(_matches: &ArgMatches, api: &mut API) -> Result<(), APIError> {
  let credits = api.get_credits()?;
  println!("{} credits", credits);

  Ok(())
}
