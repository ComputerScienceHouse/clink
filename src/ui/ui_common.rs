use crate::api::{APIError, DrinkList, Machine, Slot, API};
use crate::ui::store::{ListenerView, Store};
use cursive;
use cursive::align::{HAlign, VAlign};
use cursive::event::{Event, EventResult, Key};
use cursive::theme::{BaseColor, Color, ColorStyle, ColorType, Effect, PaletteColor, Style};
use cursive::traits::*;
use cursive::utils::span::SpannedString;
use cursive::view::{Margins, Offset, Position};

use cursive::views::{
  Dialog, DialogFocus, Layer, OnEventView, PaddedView, SelectView, ShadowView, TextView,
};
use cursive::{Cursive, CursiveRunnable};
use std::sync::{Arc, Mutex};
use std::thread;

struct ModelData {
  credits: Store<Option<i64>>,
  machines: Store<Option<DrinkList>>,
  api: API,
}

// This should really get cleaned up:
type Model = Arc<Mutex<ModelData>>;

/// Entrypoint, CLI will call this when we start up!
pub fn launch(api: API) -> Result<(), APIError> {
  api.get_token()?;
  let mut siv = cursive::default();
  let model = Arc::new(Mutex::new(ModelData {
    credits: Store::new(None),
    machines: Store::new(None),
    api,
  }));

  // Nice to have
  siv.add_global_callback('q', |s| s.quit());

  csh_logo(&mut siv);

  let padding = credit_count(Arc::clone(&model), &mut siv);

  machine_list(Arc::clone(&model), &mut siv, padding);

  let status_handle = {
    let model = Arc::clone(&model);
    let cb_sink = siv.cb_sink().clone();
    thread::spawn(move || {
      let api = model.lock().unwrap().api.clone();
      let machine_list = api.get_status_for_machine(None)?;
      let model = Arc::clone(&model);
      cb_sink
        .send(Box::new(move |siv| {
          model.lock().unwrap().machines.set(siv, Some(machine_list));
        }))
        .unwrap();
      Ok(())
    })
  };

  let credits_handle = {
    let model = Arc::clone(&model);
    let cb_sink = siv.cb_sink().clone();
    thread::spawn(move || {
      let api = model.lock().unwrap().api.clone();
      let credit_count = api.get_credits()?;
      let model = Arc::clone(&model);
      cb_sink
        .send(Box::new(move |siv| {
          model.lock().unwrap().credits.set(siv, Some(credit_count));
        }))
        .unwrap();
      Ok(())
    })
  };

  siv.run();

  status_handle.join().unwrap()?;
  credits_handle.join().unwrap()?;
  Ok(())
}

/// Draws CSH logo in the corner
fn csh_logo(siv: &mut CursiveRunnable) {
  let logo = TextView::new(SpannedString::styled(
    include_str!("./logo.txt"),
    Style::from(Effect::Dim).combine(ColorStyle::new(
      PaletteColor::HighlightText,
      PaletteColor::Background,
    )),
  ))
  .h_align(HAlign::Right)
  .v_align(VAlign::Bottom);

  siv.screen_mut().add_transparent_layer(logo.full_screen());
}

/// Draws credit counter in top-left
fn credit_count(model: Model, siv: &mut CursiveRunnable) -> Margins {
  let credit_text = TextView::empty();
  let mut listener_view = ListenerView::new(
    credit_text,
    &model.lock().unwrap().credits,
    |view, _old_credits, credits| {
      let credit_text = view.downcast_mut::<TextView>().unwrap();
      match credits {
        Some(credits) => credit_text.set_content(format!(" Credits: {} ", credits)),
        None => credit_text.set_content("Loading..."),
      };
    },
  );
  model
    .lock()
    .unwrap()
    .credits
    .use_store(siv, &mut listener_view);
  let mut dialog = Dialog::around(listener_view).padding_lrtb(0, 0, 0, 0);
  let mut size = dialog.required_size(siv.screen_size());
  let offset_y = 2;
  siv.screen_mut().add_transparent_layer_at(
    Position::new(Offset::Center, Offset::Parent(offset_y)),
    dialog,
  );

  size = size.map_y(move |y| y + (offset_y as usize) + 1);
  Margins::tb(size.y, size.y)
}

/// Draws SelectView with list of available machines
fn machine_list(model: Model, siv: &mut CursiveRunnable, padding: Margins) {
  let mut select: SelectView<Machine> = SelectView::new().h_align(HAlign::Center).autojump();

  {
    let model = Arc::clone(&model);
    select.set_on_submit(move |siv: &mut Cursive, machine: &Machine| {
      item_list(Arc::clone(&model), siv, machine.id, padding);
    });
  }

  let cb_sink = siv.cb_sink().clone();
  let mut listener_view = ListenerView::new(
    select,
    &model.lock().unwrap().machines,
    move |view, old_list, machine_list| {
      // panic rationale: failing this downcast indicates a bug in the program
      let select = view.downcast_mut::<SelectView<Machine>>().unwrap();
      select.clear();
      if let Some(machine_list) = machine_list {
        for machine in &machine_list.machines {
          select.add_item(
            SpannedString::styled(
              machine.display_name.clone(),
              match machine.is_online {
                true => Style::default(),
                false => Style::from(ColorStyle::front(ColorType::Color(Color::Light(
                  BaseColor::Red,
                ))))
                .combine(Effect::Dim)
                .combine(Effect::Bold),
              },
            ),
            machine.clone(),
          );
        }
      }
      // If we just loaded, take focus from the "Quit" button:
      if old_list.is_none() && machine_list.is_some() {
        cb_sink
          .send(Box::new(move |siv| {
            siv.call_on_all_named("machine_list_dialog", |dialog: &mut Dialog| {
              dialog.set_focus(DialogFocus::Content);
            });
          }))
          .unwrap();
      }
    },
  );
  model
    .lock()
    .unwrap()
    .machines
    .use_store(siv, &mut listener_view);

  let listener_view = OnEventView::new(listener_view)
    .on_event_inner(Key::Right, |listener_view, _event| {
      listener_view.with_child::<SelectView<Machine>, _, Option<EventResult>>(|v| {
        Some(v.on_event(Event::Key(Key::Enter)))
      })
    })
    .on_event(Key::Left, |siv| {
      siv.quit();
    });

  siv.screen_mut().add_transparent_layer(PaddedView::new(
    padding,
    ShadowView::new(Layer::new(
      Dialog::around(listener_view.scrollable())
        .title("Select a Machine")
        .button("Quit", |siv| siv.quit())
        .with_name("machine_list_dialog"),
    )),
  ));
}

/// Draws list of items available for purchase
fn item_list(model: Model, siv: &mut Cursive, machine_id: u64, padding: Margins) {
  let mut select: SelectView<Slot> = SelectView::new().h_align(HAlign::Center).autojump();
  {
    let model = Arc::clone(&model);
    select.set_on_submit(move |siv: &mut Cursive, slot: &Slot| {
      drop_drink(Arc::clone(&model), siv, slot)
    });
  }
  let select = OnEventView::new(select)
    .on_event_inner(Key::Right, move |select, _event| {
      Some(select.on_event(Event::Key(Key::Enter)))
    })
    .on_event(Key::Left, |siv| {
      siv.pop_layer();
    });
  let mut listener_view = ListenerView::new(
    select,
    &model.lock().unwrap().machines,
    move |view, _old_list, machine_list| {
      let event_view = view
        .downcast_mut::<OnEventView<SelectView<Slot>>>()
        .unwrap();
      let select = event_view.get_inner_mut();
      let machine = machine_list
        .as_ref()
        .unwrap()
        .machines
        .iter()
        .find(|machine| machine.id == machine_id)
        .unwrap();

      select.clear();
      for slot in &machine.slots {
        select.add_item(
          SpannedString::styled(
            format!("{} ({} Credits)", slot.item.name, slot.item.price),
            match !slot.active || slot.empty || slot.count.map(|c| c == 0).unwrap_or(false) {
              true => Style::from(ColorStyle::front(ColorType::Color(Color::Light(
                BaseColor::Red,
              ))))
              .combine(Effect::Dim)
              .combine(Effect::Bold),
              false => Style::default(),
            },
          ),
          slot.clone(),
        );
      }
    },
  );
  model
    .lock()
    .unwrap()
    .machines
    .use_store(siv, &mut listener_view);

  let machine_name = {
    let machines = &model.lock().unwrap().machines;
    let machines = machines.get().as_ref().unwrap();
    let machine = machines
      .machines
      .iter()
      .find(|machine| machine.id == machine_id)
      .unwrap();
    machine.display_name.clone()
  };
  siv.screen_mut().add_transparent_layer(PaddedView::new(
    padding,
    ShadowView::new(Layer::new(
      Dialog::around(listener_view.scrollable())
        .title(machine_name)
        .button("Cancel", |siv| {
          siv.pop_layer();
        }),
    )),
  ));
}

/// Fires off a drop and shows a message to the user
/// Pops off when finished
fn drop_drink(model: Model, siv: &mut Cursive, slot: &Slot) {
  let machine_id = slot.machine;
  let machine_id = model
    .lock()
    .unwrap()
    .machines
    .get()
    .as_ref()
    .unwrap()
    .machines
    .iter()
    .find(move |machine| machine.id == machine_id)
    .unwrap()
    .name
    .clone();
  let dialog = Dialog::around(TextView::new("Dropping a drink...")).title("Please Wait");
  siv.add_layer(dialog);
  let cb_sink = siv.cb_sink().clone();
  let slot_number = slot.number;
  thread::spawn(move || {
    let api = model.lock().unwrap().api.clone();
    match api.drop(machine_id, slot_number) {
      Ok(credits) => {
        let message = format!("Enjoy! You now have {} credits", credits);
        let model = Arc::clone(&model);
        let model_ref = Arc::clone(&model);
        cb_sink
          .send(Box::new(move |siv| {
            model.lock().unwrap().credits.set(siv, Some(credits));
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
        let status = api.get_status_for_machine(None).unwrap();
        cb_sink
          .send(Box::new(move |siv| {
            model_ref.lock().unwrap().machines.set(siv, Some(status));
          }))
          .unwrap();
      }
      Err(err) => {
        let message = match err {
          APIError::ServerError(_path, message) => message,
          err => format!("Couldn't drop a drink: {:?}", err),
        };
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
    };
  });
}
