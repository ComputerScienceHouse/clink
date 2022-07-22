use crate::api::{APIError, API};

pub fn list(api: &mut API, machine: Option<String>) -> Result<(), APIError> {
  let drinks = api.get_status_for_machine(machine.as_deref())?;

  for machine in drinks.machines {
    println!();
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
