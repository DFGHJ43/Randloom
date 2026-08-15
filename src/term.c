/*
 * term.c — Terminal control (ANSI escapes) + keyboard input
 *
 * term_*: clear screen, cursor positioning, show/hide cursor, VT enable
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
#else
#include <fcntl.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>
#endif

/* ══════════════════════════════════════════════════════════
 * Terminal control (ANSI escape codes)
 * ══════════════════════════════════════════════════════════ */

void term_clear(void) {
    printf("\033[2J\033[H");
}

void term_goto(int row, int col) {
    printf("\033[%d;%dH", row + 1, col + 1);
}

void term_hide_cursor(void) {
    printf("\033[?25l");
}

void term_show_cursor(void) {
    printf("\033[?25h");
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
int term_enable_vt(void) {
    HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (hOut == INVALID_HANDLE_VALUE) return -1;
    DWORD dwMode = 0;
    if (!GetConsoleMode(hOut, &dwMode)) return -1;
    dwMode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    if (!SetConsoleMode(hOut, dwMode)) return -1;
    return 0;
}
#endif

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
