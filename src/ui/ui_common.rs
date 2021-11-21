use ncurses::*;

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
