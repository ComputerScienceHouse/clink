use ncurses::*;

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
