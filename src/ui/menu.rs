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

    /* Print to the back buffer. */
    addstr("Hello, world!");

    /* Print some unicode(Chinese) string. */
    // addstr("Great Firewall dislike VPN protocol.\nGFW 不喜欢 VPN 协议。");

    /* Update the screen. */
    refresh();

    /* Wait for a key press. */
    getch();

    /* Terminate ncurses. */
    endwin();
}
