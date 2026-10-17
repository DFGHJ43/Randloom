/*
 * term.h — Terminal control & keyboard input interface
 *
 * term_* : terminal ops (Win32 console API on Windows, ANSI elsewhere)
 *          |  kb_* / read_key / KEY_* : keyboard input
 */

#ifndef TERMINAL_H
#define TERMINAL_H

/* ── Terminal control ─────────────────────────────────── */
void term_clear(void);
void term_goto(int row, int col);
void term_hide_cursor(void);
void term_show_cursor(void);
void term_flush(void);
void term_sleep_ms(int ms);
#ifdef _WIN32
/* Initialize the console output path. Called once at startup.
   Windows renders exclusively through the Win32 console API (works on
   Windows 7 through 11, in conhost and ConPTY hosts like Windows
   Terminal); no VT sequences are used, so there is one code path.
   Returns 0 on success; -1 if stdout is not attached to a console
   (redirected) — the caller should report an error and exit. */
int term_console_init(void);
void term_console_restore(void);
#endif

/* Style helpers (bold / reverse video / reset). On Windows they set the
   console text attribute; elsewhere they emit ANSI escape codes. */
void term_style_bold(void);
void term_style_reverse(void);
void term_style_reset(void);

/* ── Terminal size ────────────────────────────────────── */
/* Returns 0 on success. On failure fills *rows=24, *cols=80 and returns -1. */
int term_get_size(int *rows, int *cols);

/* Whether the output path can carry UTF-8 to the screen. The graph is drawn
   with braille patterns (U+2800..U+28FF); where that cannot be rendered the
   graph module falls back to ASCII '#' cells. Non-Windows terminals report 1;
   on Windows the console output code page decides (term_console_init tries
   to switch it to UTF-8 for the session). */
int term_utf8_supported(void);

/* ── Keyboard input ───────────────────────────────────── */
#define KEY_UP      256
#define KEY_DOWN    257
#define KEY_LEFT    258
#define KEY_RIGHT   259
#define KEY_ESC     27

int  kb_hit(void);
int  kb_get(void);
int  read_key(void);

/* Raw-mode stdin control. On Unix this disables ICANON/ECHO for the whole
   session so keys reach read_key() unedited (Backspace included); without
   it the line discipline eats Backspace and echoes keystrokes into the UI.
   No-ops on Windows, whose conio reads are raw already. Call
   term_input_init() once at startup and term_input_restore() on exit. */
void term_input_init(void);
void term_input_restore(void);

#endif /* TERMINAL_H */
