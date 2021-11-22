use ncurses::*;

use crate::ui::ui_common;
use crate::ui::inventory;

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

    mvwprintw(win, 1, 3, "SELECT A MACHINE");
    mvwprintw(win, 2, 2, "================");

    let mut api = api::API::new(); // Cheetos.
    let machines_online = api::API::get_machines(&mut api);

    match machines_online {
        Ok(machine_names) => {
            let mut machine_count = 1; // Start printing machines on the 3rd row of the Window.
            for machine in &machine_names {
                mvwprintw(win, 2 + machine_count, 2, format!("{}. {}", machine_count, machine).as_str());
                machine_count += 1;
            }

            wrefresh(win);
            refresh();
            let requested_machine = getch();
            inventory::build_menu(&mut api, requested_machine as i32 - 0x30);
            ui_common::destroy_win(win);
        },
        _ => {endwin(); panic!("You fucking idiot.");}
    }
}

