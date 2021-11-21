use ncurses::*;

use crate::ui::ui_common;

pub fn pick() {
  ui_common::launch();

  /* Status/help info. */
  addstr("Use the arrow keys to move");
  mvprintw(LINES() - 1, 0, "Press F1 to exit");
  refresh();

  /* Get the screen bounds. */
  let mut max_x = 0;
  let mut max_y = 0;
  getmaxyx(stdscr(), &mut max_y, &mut max_x);

  /* Start in the center. */
  let mut start_y = (max_y - 10) / 2;
  let mut start_x = (max_x - 10) / 2;
  let mut win = ui_common::create_win(start_y, start_x, 10, 30);
  refresh();
  getch();
  ui_common::destroy_win(win);
}

