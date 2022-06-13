use crate::api::{DrinkList, Machine, Slot, API};
use cursive;
use cursive::align::HAlign;
use cursive::traits::*;
use cursive::view::IntoBoxedView;
use cursive::views::{Dialog, OnEventView, SelectView, TextView};
use cursive::{Cursive, CursiveRunnable};
use std::sync::{Arc, Mutex};
use std::thread;

struct ModelData {
  credits: Option<u64>,
  machines: Option<DrinkList>,
  machine: Option<Machine>,
  api: API,
}

// Here we use a single mutex, but bigger models might
// prefer individual mutexes for different variables.
type Model = Arc<Mutex<ModelData>>;

pub fn launch(api: API) -> Result<(), Box<dyn std::error::Error>> {
  let mut siv = cursive::default();
  let model = Arc::new(Mutex::new(ModelData {
    credits: None,
    machines: None,
    machine: None,
    api,
  }));
  machine_list(Arc::clone(&model), &mut siv)?;
  siv.run();
  Ok(())
}

fn get_drinks(model: &Model) -> Result<DrinkList, Box<dyn std::error::Error>> {
  let mut model = model.lock().unwrap();
  match model.machines {
    Some(ref machines) => Ok(machines.clone()),
    None => {
      let machines = model.api.get_status_for_machine(None)?;
      model.machines = Some(machines.clone());
      Ok(machines)
    }
  }
}

fn get_credits(model: &Model) -> Result<u64, Box<dyn std::error::Error>> {
  let mut model = model.lock().unwrap();
  match model.credits {
    Some(credits) => Ok(credits),
    None => {
      let credits = model.api.get_credits()?;
      model.credits = Some(credits);
      Ok(credits)
    }
  }
}

fn machine_list(model: Model, siv: &mut CursiveRunnable) -> Result<(), Box<dyn std::error::Error>> {
  let mut select = SelectView::new().h_align(HAlign::Center).autojump();

  let drink_list = get_drinks(&model)?;
  for machine in drink_list.machines {
    select.add_item(machine.display_name.clone(), machine);
  }

  select.set_on_submit(move |siv: &mut Cursive, machine: &Machine| {
    item_list(Arc::clone(&model), siv, machine)
  });

  let select = OnEventView::new(select);

  siv.add_layer(
    Dialog::around(select.scrollable())
      .title("Select a Machine")
      .button("Quit", |siv| siv.quit()),
  );
  Ok(())
}

fn item_list(
  model: Model,
  siv: &mut Cursive,
  machine: &Machine,
) -> Result<(), Box<dyn std::error::Error>> {
  model.lock().unwrap().machine = Some(machine.clone());
  let mut select = SelectView::new().h_align(HAlign::Center).autojump();
  for slot in machine.slots.clone() {
    select.add_item(
      format!("{} ({} Credits)", slot.item.name, slot.item.price),
      slot,
    );
  }
  select
    .set_on_submit(move |siv: &mut Cursive, slot: &Slot| drop_drink(Arc::clone(&model), siv, slot));
  let select = OnEventView::new(select);
  siv.add_layer(
    Dialog::around(select.scrollable())
      .title(machine.display_name.clone())
      .button("Cancel", |siv| {
        siv.pop_layer();
      }),
  );
  Ok(())
}

fn drop_drink(model: Model, siv: &mut Cursive, slot: &Slot) {
  let machine_id = model.lock().unwrap().machine.as_ref().unwrap().name.clone();
  let dialog = Dialog::around(TextView::new("Dropping a drink...")).title("Please Wait");
  siv.add_layer(dialog);
  let cb_sink = siv.cb_sink().clone();
  let slot_number = slot.number;
  thread::spawn(
    move || match model.lock().unwrap().api.drop(machine_id, slot_number) {
      Ok(credits) => {
        model.lock().unwrap().credits = Some(credits);
        let message = format!("Enjoy! You now have {} credits", credits);
        cb_sink
          .send(Box::new(move |siv| {
            siv.pop_layer();
            siv.add_layer(
              Dialog::around(TextView::new(message))
                .button("Done", |siv| {
                  siv.pop_layer();
                })
                .title("Dropped Drink"),
            );
          }))
          .unwrap();
      }
      Err(err) => {
        let message = format!("Couldn't drop a drink: {:?}", err);
        cb_sink
          .send(Box::new(move |siv| {
            siv.pop_layer();
            siv.add_layer(
              Dialog::around(TextView::new(message))
                .button("Done", |siv| {
                  siv.pop_layer();
                })
                .title("Error"),
            );
          }))
          .unwrap();
      }
    },
  );
}
