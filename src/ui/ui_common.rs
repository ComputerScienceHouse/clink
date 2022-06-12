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

  /* Colors. */
  do_color();
  attron(COLOR_PAIR(2));

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

pub fn refresh_win(win: WINDOW) {
  box_(win, 0, 0);
  wrefresh(win);
}

pub fn destroy_win(win: WINDOW) {
  let ch = ' ' as chtype;
  werase(win);
  wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
  wrefresh(win);
  delwin(win);
}

pub fn draw_logo() {
  let (max_y, max_x) = get_bounds();

  mvprintw(
    max_y - 20,
    max_x - 20,
    concat!(
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
      " .........................` `....-"
    ),
  );
}

pub fn print_instructions() {
  let (max_y, max_x) = get_bounds();
  mvprintw(max_y - 3, max_x - 31, "Use the ARROW KEYS to navigate.");
  mvprintw(
    max_y - 2,
    max_x - 43,
    "ENTER to select, Q to go back, ^C to close.",
  );
}

pub fn do_color() {
  /* Start colors. */
  start_color();
  init_pair(1, COLOR_RED, COLOR_BLACK);
  init_pair(2, COLOR_WHITE, COLOR_BLACK);
}

const KEY_Q: i32 = 'q' as i32;
const KEY_NEWLINE: i32 = '\n' as i32;
const KEY_CTRL_C: i32 = 0x3;

pub enum UserInput {
  NavigateUp(i32),
  NavigateDown(i32),
  Activate(i32),
  Back(i32),
  Quit(i32),
  Unknown(i32),
}

impl From<i32> for UserInput {
  fn from(key: i32) -> Self {
    match key {
      KEY_UP => UserInput::NavigateUp(key),
      KEY_DOWN => UserInput::NavigateDown(key),
      KEY_RIGHT | KEY_NEWLINE => UserInput::Activate(key),
      KEY_LEFT | KEY_Q | KEY_BACKSPACE => UserInput::Back(key),
      KEY_CTRL_C => UserInput::Quit(key),
      key => UserInput::Unknown(key),
    }
  }
}
