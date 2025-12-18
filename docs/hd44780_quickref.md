# HD44780 quick reference (LifelineTTY)

This is a **project-focused** summary of the HD44780 controller behaviors that LifelineTTY relies on.
It is intentionally short and written in our own words.

For the full controller document that the code comments refer to, see:

- `docs/HD44780_specs.pdf`


## Hardware assumptions (as used here)

- We typically drive an HD44780-compatible LCD through a **PCF8574 I²C backpack**.
- The bus is **4-bit mode** (data lines D4–D7 only).
- On many backpacks **R/W is not wired** (often tied low), so the firmware **cannot read the busy flag** and must use conservative delays.
- Default I²C address in this repo is `0x27` (see `src/lcd_driver/mod.rs::DEFAULT_I2C_ADDR`).

## DDRAM addressing (where text lives)

The controller exposes an 80-byte Display Data RAM (DDRAM). “Rows” on the glass are mapped into DDRAM in a way that is sometimes non-linear.

### Common row base offsets

- **16×2 (primary target)**
  - Row 0 base: `0x00`
  - Row 1 base: `0x40`

- **Typical 20×4 / 16×4 mapping** (common HD44780-compatible glass)
  - Row 0 base: `0x00`
  - Row 1 base: `0x40`
  - Row 2 base: `0x00 + cols`
  - Row 3 base: `0x40 + cols`

This is the same formula used by the in-tree PCF8574 driver in `Hd44780::move_to()`.

### “Set DDRAM address”

To move the cursor, send **Set DDRAM Address** with the high bit set:

- Command byte: `0x80 | addr`

## CGRAM (custom characters / icons)

- The controller provides **64 bytes** of Character Generator RAM (CGRAM).
- In 5×8 mode, that is **8 custom characters** (slots `0..=7`).
- Each custom glyph is **8 rows** tall; each row is a byte where the **lowest 5 bits** are the pixel columns.
- To display a custom glyph, write its slot index (`0..=7`) as a character code into DDRAM.

This is why LifelineTTY’s icon system treats the 8-slot budget as a hard constraint and aggressively reuses bar/partial-block glyphs.

## Command bytes we rely on

These are the standard HD44780 command values used throughout the repo:

- Clear display: `0x01`
- Return home: `0x02`
- Entry mode set: `0x04` (increment flag is `0x02`)
- Display control: `0x08` (display on `0x04`, cursor on `0x02`, blink `0x01`)
- Function set: `0x20` (2-line flag `0x08`)
- Set CGRAM address: `0x40 | addr`
- Set DDRAM address: `0x80 | addr`

See `src/lcd_driver/mod.rs` for the exact constants used by the legacy PCF8574 driver.

## Timing notes (no busy-flag polling)

Because many backpacks cannot read BF/AC, LifelineTTY uses fixed waits.
Practical rules of thumb:

- **Clear** (`0x01`) and **Home** (`0x02`) are the “slow” operations. Use a millisecond-scale delay.
- Most other instructions are “fast” (tens of microseconds), and I²C write latency usually dominates.

In practice, the in-tree PCF8574 driver uses conservative sleeps (e.g. a few milliseconds for clear/home and ~40 µs between CGRAM writes).

## 4-bit init sequence (overview)

At a high level, a robust initialization looks like:

1. Wait after power-up (tens of milliseconds) before sending commands.
2. Send the “reset” nibbles to force a known state.
3. Switch to 4-bit mode.
4. Function set (lines/font), display off, clear/home, entry mode, then display on.

The concrete implementation is in `src/lcd_driver/mod.rs::Hd44780::new()`.

## Troubleshooting checklist

If the LCD stays blank or shows garbage:

- Check **contrast** potentiometer first (most common).
- Verify the I²C address (common values: `0x27`, `0x3f`).
- Confirm the backpack’s pin mapping matches expectations (RS/E/D4–D7/backlight wiring can vary).
- If you see intermittent corruption, increase delays around clear/home or initialization.
