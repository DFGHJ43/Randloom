/*
 * term.c — Terminal control + keyboard input
 *
 * term_*: clear screen, cursor positioning, show/hide cursor, styles.
 *   Windows renders exclusively through the Win32 console API (one path for
 *   every Windows console, 7 through 11); other platforms use ANSI escapes.
 * kb_hit/kb_get/read_key: cross-platform key reads (Windows conio / Unix termios)
 * KEY_UP/DOWN/LEFT/RIGHT/ESC: special key codes (256+)
 */

/* Strict -std=c99 (the README build command) hides POSIX extensions like
   usleep() in glibc/musl headers; GCC 14+ turns the resulting implicit
   declaration into a hard error. Must be defined before any system header.
   Harmless on Windows (and already defined by CMake's gnu99 mode). */
#ifndef _WIN32
#ifndef _DEFAULT_SOURCE
#define _DEFAULT_SOURCE
#endif
#endif

#include "term.h"

#include <stdio.h>

#ifdef _WIN32
#include <conio.h>
#include <windows.h>

/* Console output state. Windows renders exclusively through the Win32 console
   API — one code path that works on every Windows console from 7 through 11
   (conhost, Windows Terminal, ConEmu, other ConPTY hosts). */
static HANDLE g_hOut = INVALID_HANDLE_VALUE;
static WORD   g_default_attr = 0x07;            /* attrs at startup (color cmd) */
static CONSOLE_CURSOR_INFO g_orig_cursor = { 100, TRUE };
static DWORD  g_orig_mode = 0;                  /* output mode at startup */

/* Invariant: every Win32 console call that changes console state must run on
   an empty stdout buffer. printf/putchar only fill the CRT buffer while the
   console calls apply immediately; flush first so buffered text lands at the
   position/attribute that was in effect when it was produced. Otherwise
   term_flush() writes the whole frame at the final cursor position with the
   final attribute — every positioning in the frame is lost. */
static void term_sync(void) { fflush(stdout); }
#else
#include <fcntl.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>
#endif

/* ══════════════════════════════════════════════════════════
 * Terminal control (Win32 console API on Windows, ANSI elsewhere)
 * ══════════════════════════════════════════════════════════ */

void term_clear(void) {
#ifdef _WIN32
    term_sync();
    CONSOLE_SCREEN_BUFFER_INFO csbi;
    if (GetConsoleScreenBufferInfo(g_hOut, &csbi)) {
        /* Fill only the visible window (srWindow), not the scrollback —
           same semantics as ANSI \033[2J on a terminal emulator. */
        DWORD n = (DWORD)(csbi.srWindow.Bottom - csbi.srWindow.Top + 1) *
                  (DWORD)(csbi.srWindow.Right  - csbi.srWindow.Left  + 1);
        DWORD written = 0;
        COORD origin = { csbi.srWindow.Left, csbi.srWindow.Top };
        SetConsoleTextAttribute(g_hOut, g_default_attr); /* no leftover style */
        FillConsoleOutputCharacterA(g_hOut, ' ', n, origin, &written);
        SetConsoleCursorPosition(g_hOut, origin);        /* == \033[H */
    }
#else
    printf("\033[2J\033[H");
#endif
}

void term_goto(int row, int col) {
#ifdef _WIN32
    term_sync();
    CONSOLE_SCREEN_BUFFER_INFO csbi;
    if (GetConsoleScreenBufferInfo(g_hOut, &csbi)) {
        COORD pos;
        pos.X = (SHORT)(col + csbi.srWindow.Left);
        pos.Y = (SHORT)(row + csbi.srWindow.Top);
        SetConsoleCursorPosition(g_hOut, pos);
    }
#else
    printf("\033[%d;%dH", row + 1, col + 1);
#endif
}

void term_hide_cursor(void) {
#ifdef _WIN32
    term_sync();
    CONSOLE_CURSOR_INFO ci = { 1, FALSE };  /* dwSize first, then bVisible */
    SetConsoleCursorInfo(g_hOut, &ci);
#else
    printf("\033[?25l");
#endif
}

void term_show_cursor(void) {
#ifdef _WIN32
    term_sync();
    SetConsoleCursorInfo(g_hOut, &g_orig_cursor);  /* full restore, keeps size */
#else
    printf("\033[?25h");
#endif
}

void term_flush(void) {
    fflush(stdout);
}

void term_sleep_ms(int ms) {
#ifdef _WIN32
    Sleep(ms);
#else
    usleep(ms * 1000);
#endif
}

#ifdef _WIN32
/* Open the console output handle and snapshot the startup console state.
   Called once from tui_init. Windows rendering uses only the Win32 console
   API (works on Windows 7 through 11, in conhost and ConPTY hosts like
   Windows Terminal); VT is never enabled, so there is exactly one Windows
   code path. Returns 0 on success; -1 if stdout is not attached to a
   console (redirected). */
int term_console_init(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (hOut == INVALID_HANDLE_VALUE) return -1;
    DWORD dwMode = 0;
    if (!GetConsoleMode(hOut, &dwMode)) return -1;  /* stdout is not a console */
    g_hOut = hOut;

    /* Turn off ENABLE_WRAP_AT_EOL_OUTPUT (0x0002): on legacy conhost
       (Windows 7/8) writing a character at the bottom-right cell of the
       visible window — draw_box's '+' corner, written every frame —
       immediately wraps the cursor past the window and scrolls the
       viewport down one line, shifting the whole UI up one row per
       keypress and leaving ghost rows behind. This TUI positions every
       output explicitly and never relies on wrap. Restored at exit via
       atexit so the parent cmd.exe keeps wrapping long lines. */
    g_orig_mode = dwMode;
    if (SetConsoleMode(g_hOut, dwMode & ~ENABLE_WRAP_AT_EOL_OUTPUT))
        atexit(term_console_restore);

    CONSOLE_SCREEN_BUFFER_INFO csbi;
    if (GetConsoleScreenBufferInfo(g_hOut, &csbi))
        g_default_attr = csbi.wAttributes;      /* respect "color 0A" etc. */
    GetConsoleCursorInfo(g_hOut, &g_orig_cursor);
    SetConsoleTextAttribute(g_hOut, g_default_attr);
    return 0;
}

/* Restore the output mode saved by term_console_init (registered via
   atexit). The console mode is a property of the screen buffer and
   survives process exit; without this, a long line in the parent
   cmd.exe would stop wrapping at the right edge. */
void term_console_restore(void) {
    if (g_hOut != INVALID_HANDLE_VALUE && g_orig_mode != 0)
        SetConsoleMode(g_hOut, g_orig_mode);
}
#endif

void term_style_bold(void) {
#ifdef _WIN32
    term_sync();
    /* FOREGROUND_INTENSITY (0x08) affects only the foreground */
    SetConsoleTextAttribute(g_hOut, (WORD)(g_default_attr | FOREGROUND_INTENSITY));
#else
    printf("\033[1m");
#endif
}

void term_style_reverse(void) {
#ifdef _WIN32
    term_sync();
    /* Swap the foreground/background nibbles; a BACKGROUND_INTENSITY
       (0x80) moves to the foreground, like terminal emulators do. */
    SetConsoleTextAttribute(g_hOut,
        (WORD)(((g_default_attr & 0x0F) << 4) |
               ((g_default_attr & 0xF0) >> 4)));
#else
    printf("\033[7m");
#endif
}

void term_style_reset(void) {
#ifdef _WIN32
    term_sync();
    SetConsoleTextAttribute(g_hOut, g_default_attr);
#else
    printf("\033[0m");
#endif
}

/* ── Terminal size ────────────────────────────────────── */

int term_get_size(int *rows, int *cols) {
#ifdef _WIN32
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    CONSOLE_SCREEN_BUFFER_INFO csbi;
    if (hOut != INVALID_HANDLE_VALUE &&
        GetConsoleScreenBufferInfo(hOut, &csbi)) {
        /* srWindow = visible window; dwSize includes scrollback */
        *cols = csbi.srWindow.Right - csbi.srWindow.Left + 1;
        *rows = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
        if (*cols > 0 && *rows > 0)
            return 0;
    }
#else
    struct winsize ws = {0};
    /* Try stderr, stdout, stdin in order — any may be redirected */
    int fds[3] = { STDERR_FILENO, STDOUT_FILENO, STDIN_FILENO };
    for (int i = 0; i < 3; i++) {
        if (ioctl(fds[i], TIOCGWINSZ, &ws) == 0 &&
            ws.ws_col > 0 && ws.ws_row > 0) {
            *cols = ws.ws_col;
            *rows = ws.ws_row;
            return 0;
        }
    }
#endif
    /* Fallback */
    *rows = 24;
    *cols = 80;
    return -1;
}

/* ══════════════════════════════════════════════════════════
 * Keyboard input
 * ══════════════════════════════════════════════════════════ */

#ifdef _WIN32
int kb_hit(void) { return _kbhit(); }
int kb_get(void) { return _getch(); }
#else
int kb_hit(void) {
    struct termios oldt, newt;
    int ch, oldf;
    tcgetattr(STDIN_FILENO, &oldt);
    newt = oldt;
    newt.c_lflag &= (unsigned int)~(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &newt);
    oldf = fcntl(STDIN_FILENO, F_GETFL, 0);
    fcntl(STDIN_FILENO, F_SETFL, oldf | O_NONBLOCK);
    ch = getchar();
    tcsetattr(STDIN_FILENO, TCSANOW, &oldt);
    fcntl(STDIN_FILENO, F_SETFL, oldf);
    if (ch != EOF) { ungetc(ch, stdin); return 1; }
    return 0;
}

int kb_get(void) {
    struct termios oldt, newt;
    tcgetattr(STDIN_FILENO, &oldt);
    newt = oldt;
    newt.c_lflag &= (unsigned int)~(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &newt);
    int ch = getchar();
    tcsetattr(STDIN_FILENO, TCSANOW, &oldt);
    return ch;
}
#endif

int read_key(void) {
    int c = kb_get();
    if (c == 0 || c == 224) {  /* extended key prefix (Windows) */
        int c2 = kb_get();
        switch (c2) {
        case 72: return KEY_UP;
        case 80: return KEY_DOWN;
        case 75: return KEY_LEFT;
        case 77: return KEY_RIGHT;
        default: return c2;
        }
    }
    if (c == 27) {  /* ESC or ANSI escape sequence */
        if (kb_hit()) {
            int c2 = kb_get();
            if (c2 == '[') {
                int c3 = kb_get();
                switch (c3) {
                case 'A': return KEY_UP;
                case 'B': return KEY_DOWN;
                case 'C': return KEY_RIGHT;
                case 'D': return KEY_LEFT;
                default:  return c3;
                }
            }
        }
        return KEY_ESC;
    }
    if (c == '\r') c = '\n';
    return c;
}
