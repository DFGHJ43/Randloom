/*
 * chars.h — UTF-8 byte sequences for the block / box-drawing glyph set
 *
 * The UI paints its frame, axes and graph fills with the Unicode Block
 * Elements and Box Drawing characters that terminal UIs usually prefer:
 *
 *   full block : █  (U+2588)
 *   half block : ▀  (U+2580) upper half, ▄  (U+2584) lower half
 *   box drawing: ─ │ ┌ ┬ ┐ └ ┴ ┘ ├ ┤ ┼  (U+2500..U+254C family)
 *   arrow      : →  (U+2192, x-axis direction)
 *
 * They are spelled as explicit UTF-8 byte sequences, so the string literals
 * stay plain ASCII and cannot be corrupted by a compiler that assumes a
 * legacy source encoding (MSVC, unlike GCC/Clang, has no fixed UTF-8 default).
 * The glyphs in the comments are documentation only.
 *
 * All of these characters occupy one terminal cell, so callers keep using
 * ordinary column arithmetic for layout.
 */

#ifndef CHARS_H
#define CHARS_H

/* ── Box drawing ──────────────────────────────────────── */
#define UTF8_HBAR   "\xE2\x94\x80" /* ─ */
#define UTF8_VBAR   "\xE2\x94\x82" /* │ */
#define UTF8_TL     "\xE2\x94\x8C" /* ┌ */
#define UTF8_TM     "\xE2\x94\xAC" /* ┬ */
#define UTF8_TR     "\xE2\x94\x90" /* ┐ */
#define UTF8_ML     "\xE2\x94\x9C" /* ├ */
#define UTF8_MR     "\xE2\x94\xA4" /* ┤ */
#define UTF8_BL     "\xE2\x94\x94" /* └ */
#define UTF8_BM     "\xE2\x94\xB4" /* ┴ */
#define UTF8_BR     "\xE2\x94\x98" /* ┘ */
#define UTF8_CROSS  "\xE2\x94\xBC" /* ┼ */

/* ── Block elements ───────────────────────────────────── */
#define UTF8_FULL_BLOCK  "\xE2\x96\x88" /* █ */
#define UTF8_UPPER_HALF  "\xE2\x96\x80" /* ▀ */
#define UTF8_LOWER_HALF  "\xE2\x96\x84" /* ▄ */

/* ── Misc ─────────────────────────────────────────────── */
#define UTF8_ARROW  "\xE2\x86\x92" /* → */

#endif /* CHARS_H */
