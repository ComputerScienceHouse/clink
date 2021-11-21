use ncurses::*;

use crate::ui::ui_common;

use crate::api;

struct Item {
    name: String,
    price: i32
}

impl Item {
    pub fn new(name: String, price: i32) -> Self {
        Self {
            name,
            price,
        }
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

    mvwprintw(win, 1, 3, "SELECT A DRINK");
    mvwprintw(win, 2, 2, "================");

//    let machines_online = api::get_inventory(api, machine_index);

    // Dummy function because the above is broken. Also I was rly tired when I wrote the below code
    // so all the variable names are wrong lolololol
    let machines_online = get_inventory();

    let mut machines = 1; // Start printing machines on the 3rd row of the Window.
    for machine in &machines_online {
        mvwprintw(win, 2 + machines, 2, format!("{}. {} ({})", machines, machine.name, machine.price).as_str());
        machines += 1;
    }

    mvwprintw(win, height-3, width-20, "Credits: 69420");

//    mvwprintw(win, 3, 5, "tits");

    wrefresh(win);
    refresh();
    let requested_machine = getch();
/*    match requested_machine as i32 - 0x30 {
        1 => panic!("Damb fuck, sheeeeesh."),
        2 => panic!("Damb fuck, sheeeeesh."),
        3 => panic!("Damb fuck, sheeeeesh."),
        _=> panic!("Dude, fucking seriously?")
    }
*/

    ui_common::destroy_win(win);
}

fn get_inventory() -> Vec<Item> {
    vec![Item {name: "Coke".to_string(), price: 10}, Item {name: "Morning Brew".to_string(), price: 25}, Item {name: "ligma".to_string(), price: 69}]
}
