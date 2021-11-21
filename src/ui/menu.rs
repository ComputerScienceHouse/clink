use ncurses::*;

use crate::ui::ui_common;

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
  let mut win = ui_common::create_win(start_y, start_x, 10, 10);

  let mut ch = getch();
  while ch != KEY_F(1)
  {
    match ch
    {
      KEY_LEFT =>
      {
        start_x -= 1;
        ui_common::destroy_win(win);
        win = ui_common::create_win(start_y, start_x, 10, 10);
      },
      KEY_RIGHT =>
      {
        start_x += 1;
        ui_common::destroy_win(win);
        win = ui_common::create_win(start_y, start_x, 10, 10);
      },
      KEY_UP =>
      {
        start_y -= 1;
        ui_common::destroy_win(win);
        win = ui_common::create_win(start_y, start_x, 10, 10);
      },
      KEY_DOWN =>
      {
        start_y += 1;
        ui_common::destroy_win(win);
        win = ui_common::create_win(start_y, start_x, 10, 10);
      },
      _ => { }
    }
    ch = getch();
  }
  endwin();
}

