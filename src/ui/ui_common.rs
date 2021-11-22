use ncurses::*;

pub fn get_bounds() -> (i32, i32) {
    /* Get the screen bounds. */
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);
    (max_y, max_x)
}

pub fn launch() {
    /* Setup ncurses. */
    initscr();
    raw();

    /* Allow for extended keyboard (like F1). */
    keypad(stdscr(), true);
    noecho();

    /* Invisible cursor. */
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    /* Update the screen. */
    refresh();
}

pub fn end() {
    endwin();
}

pub fn create_win(y: i32, x: i32, height: i32, width: i32) -> WINDOW {
  let win = newwin(height, width, y, x);
  box_(win, 0, 0);
  wrefresh(win);
  win
}

pub fn destroy_win(win: WINDOW) {
  let ch = ' ' as chtype;
        wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
        wrefresh(win);
        delwin(win);
}

pub fn draw_logo() {
    let (max_y, max_x) = get_bounds();

    mvprintw(max_y-20, max_x-20, concat!(
    "\n'{tttttttttttttttttttttttt^ *tttt\\ \n",
    ":@@@@@@@@@@@@@@@@@@@@@@@@@m d@@@@N`\n",
    ":@@@@@@@@@@@@@@@@@@@@@@@@@m d@@@@N`\n",
    ":@@@@@m:::::::::::::rQ@@@@m d@@@@N`\n",
    ":@@@@@] vBBBBBBBBBN,`]oooo* d@@@@N`\n",
    ":@@@@@] o@@@NNNQ@@@\"`ueeee| d@@@@N`\n",
    ":@@@@@] o@@&   ,||?`'Q@@@@m d@@@@N`\n",
    ":@@@@@] o@@Q]tt{{{z-'Q@@@@QOQ@@@@N`\n",
    ":@@@@@] o@@@@@@@@@@\"'Q@@@@@@@@@@@N`\n",
    ":@@@@@] ';;;;;;y@@@\"'Q@@@@N7Q@@@@N`\n",
    ":@@@@@] \\KKe^^^a@@@\"'Q@@@@m d@@@@N`\n",
    ":@@@@@] o@@@@@@@@@@\" _::::' d@@@@N`\n",
    ":@@@@@] raaaaaaaaay..H####} d@@@@N`\n",
    ":@@@@@#eeeeeeeeeeeeek@@@@@m d@@@@N`\n",
    ":@@@@@@@@@@@@@@@@@@@@@@@@@m d@@@@N`\n",
    ":@@@@@@@@@@@@@@@@@@@@@@@@@e K@@@@W`\n",
    " .........................` `....-"));
}
