extern crate term;
extern crate vga_framebuffer;

use vga_framebuffer::{AsciiConsole, Attr, Col, Colour, Position, Row};

mod rust_logo;

struct Dummy;

impl<'a> vga_framebuffer::Hardware for &'a mut Dummy {
    fn configure(&mut self, width: u32, sync_end: u32, line_start: u32, clock_rate: u32) {
        println!(
            "width={}, sync_end={}, line_start={}, clock_rate={}",
            width, sync_end, line_start, clock_rate
        );
    }

    /// Called when V-Sync needs to be high.
    fn vsync_on(&self) {
        println!("vsync_on");
    }

    /// Called when V-Sync needs to be low.
    fn vsync_off(&self) {
        println!("vsync_off");
    }
}

use std::fmt::Write;

fn main() {
    let mut mode2_buffer = vec![
        0xAAu8;
        vga_framebuffer::USABLE_HORIZONTAL_OCTETS
            * vga_framebuffer::USABLE_LINES_MODE2
    ];
    let mut d = Dummy;
    let mut fb = vga_framebuffer::FrameBuffer::new();
    let max_col = Col(vga_framebuffer::TEXT_MAX_COL as u8);
    let max_row = Row(vga_framebuffer::TEXT_MAX_ROW as u8);
    fb.init(&mut d);
    fb.clear();
    fb.write_char_at(b'$', Position::origin()).unwrap();
    fb.write_char_at(b'$', Position::new(max_row, Col::origin()))
        .unwrap();
    fb.write_char_at(b'$', Position::new(Row::origin(), max_col))
        .unwrap();
    fb.write_char_at(b'$', Position::new(max_row, max_col))
        .unwrap();
    writeln!(fb, "\nThis is a test").unwrap();
    render_page(&fb);

    let mut wheel = [Colour::Red, Colour::Green, Colour::Blue].iter().cycle();
    for y in 0..=vga_framebuffer::TEXT_MAX_ROW {
        for x in 0..=vga_framebuffer::TEXT_MAX_COL {
            fb.set_attr_at(
                Position::new(Row(y as u8), Col(x as u8)),
                Attr::new(Colour::White, *wheel.next().unwrap()),
            );
        }
        wheel.next();
    }

    for (src, dest) in rust_logo::RUST_LOGO_DATA
        .iter()
        .zip(mode2_buffer.iter_mut())
    {
        // Our source is an X-Bitmap, which puts the pixels in LSB-first order.
        // We need MSB first order for Monotron.
        *dest = flip_byte(*src);
    }

    // Attach a graphical buffer at a scan-line. It is interpreted as
    // being a grid 48 bytes wide and as long as given. Each line
    // is output twice. We've attached it to the first scan-line.
    fb.mode2(&mut mode2_buffer[..], 0);

    render_page(&fb);

    let _ = fb.mode2_release();

    fb.clear();

    fb.set_custom_font(Some(&vga_framebuffer::freebsd_teletext::FONT_DATA));

    writeln!(fb, "This is teletext").unwrap();
    for ch in 0x80..=0xFF {
        fb.write_char(ch, None);
    }

    render_page(&fb);

    fb.set_custom_font(None);

    fb.clear();
    // You have to put double-height text in twice, once for the top line and once for the bottom line.
    writeln!(fb, "\u{001b}^\u{001b}k\u{001b}RThis is double height text").unwrap();
    writeln!(fb, "\u{001b}v\u{001b}k\u{001b}GThis is double height text").unwrap();
    writeln!(fb, "\u{001b}-\u{001b}k\u{001b}WThis is normal height text").unwrap();

    render_page(&fb);
}

fn render_page<T>(fb: &vga_framebuffer::FrameBuffer<T>) where T: vga_framebuffer::Hardware {
    for _r in 0..628 {
        render_line(fb);
    }
}

fn render_line<T>(fb: &vga_framebuffer::FrameBuffer<T>) where T: vga_framebuffer::Hardware
{
    let mut old_colour = None;
    let mut t = term::stdout().unwrap();
    fb.isr_sol();
    for (red, green, blue) in fb.iter_u8() {
        for bit in (0..8).rev() {
            let red_bit = red & (1 << bit) != 0;
            let blue_bit = blue & (1 << bit) != 0;
            let green_bit = green & (1 << bit) != 0;
            let colour: u8 = ((red_bit as u8) << 2) + ((green_bit as u8) << 1) + (blue_bit as u8);
            if old_colour != Some(colour) {
                match colour {
                    0b110 => {
                        t.fg(term::color::YELLOW).unwrap();
                    }
                    0b101 => {
                        t.fg(term::color::MAGENTA).unwrap();
                    }
                    0b100 => {
                        t.fg(term::color::RED).unwrap();
                    }
                    0b011 => {
                        t.fg(term::color::CYAN).unwrap();
                    }
                    0b010 => {
                        t.fg(term::color::GREEN).unwrap();
                    }
                    0b001 => {
                        t.fg(term::color::BLUE).unwrap();
                    }
                    0b000 => {
                        t.fg(term::color::BLACK).unwrap();
                    }
                    _ => {
                        t.fg(term::color::WHITE).unwrap();
                    }
                }
                old_colour = Some(colour);
            }
            write!(t, "█").unwrap();
        }
    }
    writeln!(t).unwrap();
}

fn flip_byte(mut b: u8) -> u8 {
    b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
    b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
    (b & 0xAA) >> 1 | (b & 0x55) << 1
}
