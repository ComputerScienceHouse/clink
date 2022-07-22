use crate::api::{APIError, API};

pub fn token(api: &mut API) -> Result<(), APIError> {
  println!("{}", api.get_token()?);

  Ok(())
}
