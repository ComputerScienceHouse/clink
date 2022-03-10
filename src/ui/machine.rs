use ncurses::*;
use crate::ui::inventory;
use crate::ui::ui_common;
use serde_json::{Map, Value};
use crate::api::APIError;
use crate::api;

pub fn pick_machine(api: &mut api::API) -> Result<(), Box<dyn std::error::Error>> {
  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);
  /* Start in the center. */
  let height = 10;
  let width = 30;
  let start_y = (max_y - height) / 2;
  let start_x = (max_x - width) / 2;
  let win = ui_common::create_win(start_y, start_x, height, width);

  // The API needs a sec...
  mvwprintw(win, 1, 2, "Loading...");
  ui_common::refresh_win(win);

  // I wanna draw the menu _over_ the logo, so that comes first.
  ui_common::draw_logo();
  ui_common::print_instructions();
  box_(win, 0, 0);
  mvwprintw(win, 1, 2, "SELECT A MACHINE");
  mvwprintw(win, 2, 2, "================");

  let mut credits = match api::API::get_credits(api) {
    Ok(credits) => credits,
    Err(err) => {
        eprintln!("{}", err);
        return Err(Box::new(APIError::Unauthorized));
    }
  };
  mvwprintw(win, height - 2, width - 20, format!("Credits: {}", credits).as_str());

  let machine_status = match api::API::get_machine_status(api) {
      Ok(status) => {
          status
      },
      Err(fuck) => {
        ui_common::destroy_win(win);
        ui_common::end();
        panic!("Error: Could not query machine status ({})", fuck)
      }
  };
  let machines_online = parse_machines(&machine_status);
  refresh();
  ui_common::refresh_win(win);
  match machines_online {
    Ok(machine_names) => {
      let machine_count = machine_names.len();
      let mut selected_machine: i32 = 0;
      for n in 0..machine_count {
        if n as i32 == selected_machine {
          wattron(win, A_REVERSE());
        }
        mvwprintw(
          win, 3 + n as i32, 2,
          machine_names[n].as_str()
        );
        wattroff(win, A_REVERSE());
      }     
      ui_common::refresh_win(win); 
      let mut key = getch();
      loop {
        match key {
            KEY_UP => {
                if selected_machine > 0 {
                    selected_machine -= 1;
                }
            },
            KEY_DOWN => {
                if selected_machine < machine_count as i32 - 1 {
                    selected_machine += 1;
                }
            },
            KEY_RIGHT => {
                inventory::build_menu(api, &machine_status, selected_machine);
                // Refresh credits in case we bought anything.
                credits = match api::API::get_credits(api) {
                    Ok(credits) => credits,
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err(Box::new(APIError::Unauthorized));
                    }
                  };
                wmove(win, height-2, width-20);
                wclrtoeol(win);
                mvwprintw(win, height-2, width-20, format!("Credits: {}", credits).as_str());
                box_(win, 0, 0);
                refresh();
                wrefresh(win);
            },
            KEY_LEFT => {
                break;
            },
            _ => {
                refresh();
            }
        }
        for n in 0..machine_count {
          if n as i32 == selected_machine {
            wattron(win, A_REVERSE());
          }
          mvwprintw(
            win, 3 + n as i32, 2,
            machine_names[n].as_str()
          );
          wattroff(win, A_REVERSE());
        }

        ui_common::refresh_win(win); 
        key = getch(); 
      }
      ui_common::destroy_win(win);
      Ok(())
    }
    _ => {
      endwin();
      panic!("Could not fetch active machines.");
    }
  }
}

pub fn parse_machines(status: &Value) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let mut display_names = Vec::new();
  let drinks: &Map<String, Value> = match status.as_object() {
    Some(drinks) => drinks,
            None => panic!("No status object found."),
  };
  let machines: &Vec<Value> = match drinks["machines"].as_array() {
    Some(machines) => machines,
              None => panic!("No machine array found."),
  };
  for machine in machines {
    //let machine: &Map<String, Value> =
    match machine.as_object() {
      Some(machine) => display_names.push(machine["display_name"].as_str().unwrap().to_string()),
               None => panic!("No machines found."),
    };
  }
  return Ok(display_names);
}

