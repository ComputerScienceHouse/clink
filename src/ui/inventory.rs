use ncurses::*;

use crate::ui::ui_common;

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

pub fn build_menu(api: &mut api::API, machine_index: i32) {
  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);

  /* Create the window */
  let height = 30;
  let width = 50;
  let start_y = (max_y - height) / 2;
  let start_x = (max_x - width) / 2;
  let win = ui_common::create_win(start_y, start_x, height, width);

  // Usually the UI needs a second to fetch from the API. Whoops lol
  mvwprintw(win, 1, 3, "Loading...");
  wrefresh(win);

  mvwprintw(win, 1, 3, "SELECT A DRINK");
  mvwprintw(win, 2, 2, "================");

  let inventory = api::API::get_inventory(api, machine_index);
  match inventory {
    Ok(slots) => {
     /* let mut slot_count = 1; // Start printing machines on the 3rd row of the Window.
      for slot in slots.iter() {
        mvwprintw(
          win, 2 + slot_count, 2,
          format!("{}. {} ({})", slot_count, slot.name, slot.price).as_str(),
        );
        slot_count += 1;
      }*/
      // TODO: Get real amt of credits.
      let credits = api::API::get_credits(api);
      mvwprintw(win, height - 3, width - 20, format!("Credits: {}", credits.unwrap()).as_str());
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
                  vend(api, selected_slot);
                }
                else {
                  deny();
                }
            },
            _ => {
                ui_common::destroy_win(win);
                return;
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

pub fn vend(api: &mut api::API, slot_index: i32) {
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

  mvwprintw(win, 1, 3, "Vending! (Not really)");
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
  mvwprintw(win, 1, 3, "Shit, dude, that's empty.");
  mvwprintw(win, 3, 3, "Press any key to continue");
  wrefresh(win);
  refresh();
  getch();
  wattroff(win, COLOR_PAIR(1));
  ui_common::destroy_win(win);
  attroff(COLOR_PAIR(1));
}
