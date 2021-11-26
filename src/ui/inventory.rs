use ncurses::*;
use crate::ui::ui_common;
use serde_json::{Map, Value};
use crate::api;

#[derive(Debug)]
pub struct Item {
  pub name: String,
  pub price: i32,
  pub empty: bool,
}

impl Item {
  pub fn new(name: String, price: i32, empty: bool) -> Self {
    Self { name, price, empty }
  }
}

pub fn build_menu(api: &mut api::API, machine_status: &Value, machine_index: i32) {
  /* Get the screen bounds. */
  let (max_y, max_x) = ui_common::get_bounds();

  /* Create the window */
  let height = 30;
  let width = 50;
  let start_y = (max_y - height) / 2;
  let start_x = (max_x - width) / 2;
  let win = ui_common::create_win(start_y, start_x, height, width);

  // Usually the UI needs a second to fetch from the API. Whoops lol
  mvwprintw(win, 1, 3, "Loading...");
  wrefresh(win);

  let machine_name = parse_machine_name(&machine_status, machine_index).unwrap();
  let inventory = parse_inventory(&machine_status, machine_index);

  mvwprintw(win, 1, 3, format!("{} -> SELECT A DRINK", machine_name).as_str());
  mvwprintw(win, 2, 2, "==========================");

  match inventory {
    Ok(slots) => {
      // TODO: Get real amt of credits.
      let credits = api::API::get_credits(api);
      mvwprintw(win, height - 2, width - 20, format!("Credits: {}", credits.unwrap()).as_str());
      wrefresh(win);
      refresh();
      //let requested_machine = getch();
      //TODO: something. Drop drink I guess.
    
      let slot_count = slots.len();
      let mut selected_slot: i32 = 0;

      for n in 0..slot_count {
          if n as i32 == selected_slot {
              wattron(win, A_REVERSE());
          }
          if slots[n].empty {
            //wattron(win, A_DIM());
            wattron(win, COLOR_PAIR(1));
          }
          mvwprintw(
              win, 3 + n as i32, 2,
              format!("{} ({} credits)", slots[n].name, slots[n].price).as_str(),
          );
          //wattroff(win, A_DIM());
          wattroff(win, COLOR_PAIR(1));
          wattroff(win, A_REVERSE());
      }

      refresh();
      wrefresh(win);
    
      let mut key = getch();
      loop {
        match key {
            KEY_UP => {
              if selected_slot > 0 {
                selected_slot -= 1;
              }
            },
            KEY_DOWN => {
              if selected_slot < slot_count as i32 - 1 {
                selected_slot += 1;
              }
            },
            KEY_RIGHT => { 
              //inventory::build_menu(&mut api, selected_machine);
              if !slots[selected_slot as usize].empty {
                match api.drop(machine_name.clone(), selected_slot as u8) {
                    Ok(()) => vend(),
                    _    => deny()
                }
              }
              else {
                deny();
              }
            },
            KEY_LEFT => {
              ui_common::destroy_win(win);
              return;
            },
            _ => {
              refresh();
            }
        }
        
        for n in 0..slot_count {
          if n as i32 == selected_slot {
            wattron(win, A_REVERSE());
          }
          if slots[n].empty {
            //wattron(win, A_DIM());
            wattron(win, COLOR_PAIR(1));
          }
          mvwprintw(
            win, 3 + n as i32, 2,
            format!("{} ({} credits)", slots[n].name, slots[n].price).as_str(),
          );
          //wattroff(win, A_DIM());
          wattroff(win, COLOR_PAIR(1));
          wattroff(win, A_REVERSE());
        }

        refresh();
        wrefresh(win);

        key = getch(); 
      }
    },
    _ => {
        endwin();
        panic!("Error: Could not query inventory");
    }
  }
}

pub fn vend() {
  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);

  /* Create the window */
  let height = 5;
  let width = 40;
  let start_y = (max_y - height) / 2;
  let start_x = (max_x - width) / 2;
  let win = ui_common::create_win(start_y, start_x, height, width);

  mvwprintw(win, 1, 3, "Item Dropped!");
  mvwprintw(win, 3, 3, "Press any key to continue");
  wrefresh(win);
  refresh();
  getch();
  ui_common::destroy_win(win);
}

pub fn deny() {
  attron(COLOR_PAIR(1));
  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);

  /* Create the window */
  let height = 5;
  let width = 40;
  let start_y = (max_y - height) / 2;
  let start_x = (max_x - width) / 2;
  let win = ui_common::create_win(start_y, start_x, height, width);

  wattron(win, COLOR_PAIR(1));
  mvwprintw(win, 1, 3, "Slot empty or insufficient funds.");
  mvwprintw(win, 3, 3, "Press any key to continue");
  wrefresh(win);
  refresh();
  getch();
  wattroff(win, COLOR_PAIR(1));
  ui_common::destroy_win(win);
  attroff(COLOR_PAIR(1));
}

pub fn parse_inventory(
  status: &Value,
  machine_index: i32,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
  // TODO: There's a better way to handle these. You could just
  // Unwrap them, or do something else.
  let drinks: &Map<String, Value> = match status.as_object() {
    Some(drinks) => drinks,
    None => panic!("Fuck"),
  };

  let machines: &Vec<Value> = match drinks["machines"].as_array() {
    Some(machines) => machines,
    None => panic!("Fuck"),
  };

  let selected_machine = machines[machine_index as usize].clone();
  let mut slots: Vec<Item> = Vec::new();
  for object in selected_machine["slots"].as_array().unwrap() {
      
    let empty: bool = match object["item"]["name"].as_str() {
      Some("Empty") => true,
      _ => false
    };
    slots.push(Item {
      name: object["item"]["name"].to_string(),
      price: object["item"]["price"].as_i64().unwrap() as i32,
      empty: empty
    });
  }
  return Ok(slots);
}

pub fn parse_machine_name(
    status: &Value,
    machine_index: i32,
) -> Result<String, Box<dyn std::error::Error>> {
  let drinks: &Map<String, Value> = match status.as_object() {
    Some(drinks) => drinks,
    None => panic!("Fuck"),
  };

  let machines: &Vec<Value> = match drinks["machines"].as_array() {
    Some(machines) => machines,
    None => panic!("Fuck"),
  };
  Ok(machines[machine_index as usize]["name"].as_str().unwrap().to_string())
}
