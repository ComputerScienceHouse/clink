use ncurses::*;

use crate::ui::inventory;
use crate::ui::ui_common;

use crate::api;

pub fn pick() {
  ui_common::launch();

  refresh();

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
  mvwprintw(win, 1, 3, "Loading...");
  wrefresh(win);

  // I wanna draw the menu _over_ the logo, so that comes first.
  ui_common::draw_logo();
  ui_common::print_instructions();
  box_(win, 0, 0);

  mvwprintw(win, 1, 2, "SELECT A MACHINE");
  mvwprintw(win, 2, 2, "================");

  let mut api = api::API::new(); // Cheetos.
  let machines_online = api::API::get_machines(&mut api);

  match machines_online {
    Ok(machine_names) => {
      let machine_count = machine_names.len(); // Start printing machines on the 3rd row of the Window.
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

      refresh();
      wrefresh(win);
    
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
                inventory::build_menu(&mut api, selected_machine);
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

        refresh();
        wrefresh(win);

        key = getch(); 
      }

      //inventory::build_menu(&mut api, requested_machine as i32 - 0x30 - 1); // -1 to start at zero
      ui_common::destroy_win(win);
    }
    _ => {
      endwin();
      panic!("Could not fetch active machines.");
    }
  }
}
