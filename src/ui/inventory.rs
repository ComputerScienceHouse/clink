use ncurses::*;

use crate::ui::ui_common;

use crate::api;

#[derive(Debug)]
pub struct Item {
  pub name: String,
  pub price: i32,
}

impl Item {
  pub fn new(name: String, price: i32) -> Self {
    Self { name, price }
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
      let mut slot_count = 1; // Start printing machines on the 3rd row of the Window.
      for slot in slots.iter() {
        mvwprintw(
          win, 2 + slot_count, 2,
          format!("{}. {} ({})", slot_count, slot.name, slot.price).as_str(),
        );
        slot_count += 1;
      }
      // TODO: Get real amt of credits.
      let credits = api::API::get_credits(api);
      mvwprintw(win, height - 3, width - 20, format!("Credits: {}", credits.unwrap()).as_str());
      wrefresh(win);
      refresh();
      let requested_machine = getch();
      //TODO: something. Drop drink I guess.
      ui_common::destroy_win(win);
    },
    _ => {
        endwin();
        panic!("Error: Could not query inventory");
    }
  }
}
