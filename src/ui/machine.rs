use ncurses::*;

use crate::ui::ui_common;
use crate::ui::inventory;

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

    mvwprintw(win, 1, 3, "SELECT A MACHINE");
    mvwprintw(win, 2, 2, "================");

    let machines_online = get_machines();

    let mut machines = 1; // Start printing machines on the 3rd row of the Window.
    for machine in &machines_online {
        mvwprintw(win, 2 + machines, 2, format!("{}. {}", machines, machine).as_str());
        machines += 1;
    }

//    mvwprintw(win, 3, 5, "tits");

    wrefresh(win);
    refresh();
    let requested_machine = getch();
    match requested_machine as i32 - 0x30 {
        1 => inventory::build_menu(),
        2 => inventory::build_menu(),
        3 => inventory::build_menu(),
        _=> panic!("Dude, fucking seriously?")
    }
    ui_common::destroy_win(win);
}

fn get_machines() -> Vec<String> {
    vec!["Big Drink".to_string(), "Little Drink".to_string(), "Snack".to_string()]
}
