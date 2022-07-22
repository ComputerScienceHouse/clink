use crate::api::{APIError, API};

pub fn drop(api: &mut API, machine: String, slot: u8) -> Result<(), APIError> {
  let credits = api.drop(machine, slot)?;
  println!("Item dropped! Your new balance is {}", credits);
  Ok(())
}
