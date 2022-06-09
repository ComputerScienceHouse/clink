use crate::api;
use crate::ui::ui_common;
use ncurses::*;

#[derive(Debug)]
pub struct Item {
  pub name: String,
  pub price: i32,
  pub active: bool,
  pub empty: bool,
}

pub fn build_menu(api: &mut api::API, machine_status: &api::DrinkList, machine_index: usize) {
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

  let machine = &machine_status.machines[machine_index];

  mvwprintw(
    win,
    1,
    3,
    format!("{} -> SELECT A DRINK", machine.display_name).as_str(),
  );
  mvwprintw(win, 2, 2, "==========================");

  // TODO: Get real amt of credits.
  let mut credits = api::API::get_credits(api).unwrap();
  mvwprintw(
    win,
    height - 2,
    width - 20,
    format!("Credits: {}", credits).as_str(),
  );
  wrefresh(win);
  refresh();
  //let requested_machine = getch();
  //TODO: something. Drop drink I guess.

  let slots = &machine.slots;

  let slot_count = slots.len();
  let mut selected_slot: i32 = 0;

  for (n, slot) in slots.iter().enumerate() {
    if n as i32 == selected_slot {
      wattron(win, A_REVERSE());
    }
    if slot.empty {
      wattron(win, COLOR_PAIR(1));
    }
    if !slot.active {
      wattron(win, A_DIM());
    }
    mvwprintw(
      win,
      3 + n as i32,
      2,
      format!("{} ({} credits)", slot.item.name, slot.item.price).as_str(),
    );
    wattroff(win, A_DIM());
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
      }
      KEY_DOWN => {
        if selected_slot < slot_count as i32 - 1 {
          selected_slot += 1;
        }
      }
      KEY_RIGHT => {
        //inventory::build_menu(&mut api, selected_machine);
        if !slots[selected_slot as usize].empty && slots[selected_slot as usize].active {
          match api.drop(machine.name.clone(), selected_slot as u8 + 1) {
            // The API returns a zero-indexed array of slots, but Mizu wants it to be 1-indexed
            Ok(new_credits) => {
              credits = new_credits;
              vend();
              wmove(win, height - 2, width - 20);
              wclrtoeol(win);
              mvwprintw(
                win,
                height - 2,
                width - 20,
                format!("Credits: {}", credits).as_str(),
              );
            }
            _ => deny(),
          }
        } else {
          deny();
        }
      }
      KEY_LEFT => {
        ui_common::destroy_win(win);
        return;
      }
      _ => {
        refresh();
      }
    }

    for (n, slot) in slots.iter().enumerate() {
      if n as i32 == selected_slot {
        wattron(win, A_REVERSE());
      }
      if slot.empty {
        wattron(win, COLOR_PAIR(1));
      }
      if !slot.active {
        wattron(win, A_DIM());
      }
      mvwprintw(
        win,
        3 + n as i32,
        2,
        format!("{} ({} credits)", slot.item.name, slot.item.price).as_str(),
      );
      wattroff(win, A_DIM());
      wattroff(win, COLOR_PAIR(1));
      wattroff(win, A_REVERSE());
    }

    refresh();
    wrefresh(win);

    key = getch();
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
