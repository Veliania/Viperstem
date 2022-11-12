//use crate::boot_info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use limine::{LimineFramebufferResponse, LimineFramebuffer, LimineFramebufferRequest};
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;
use spinning_top::Spinlock;

/// The global logger instance used for the `log` crate.
pub static mut LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

/// A [`Logger`] instance protected by a spinlock.
pub struct LockedLogger(Spinlock<Logger>);

/// Additional vertical space between lines
const LINE_SPACING: usize = 0;
/// Additional vertical space between separate log messages
const _LOG_SPACING: usize = 2;

pub static mut FRAME_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

pub fn init() {
    unsafe {
        LOGGER = OnceCell::new(LockedLogger::new(FRAME_REQUEST.get_response().get().expect("Could not get frame buffer") as *const LimineFramebufferResponse));
    }
}

pub fn set_colors(back: u32, fore: u32) {
    unsafe {
        LOGGER.get().as_deref().unwrap().0.lock().screen_color = back;
        LOGGER.get().as_deref().unwrap().0.lock().text_color = fore;

        LOGGER.get().as_deref().unwrap().0.lock().framebuffer.fill(back);
    }
}

impl LockedLogger {
    /// Create a new instance that logs to the given framebuffer.
    pub fn new(frame_buffer: *const LimineFramebufferResponse) -> LockedLogger {
        unsafe {
            let ptr = (*frame_buffer).framebuffers.address.as_ptr().unwrap() as *mut u32;
            let len = (*frame_buffer).framebuffers.size();
            let frame = core::slice::from_raw_parts_mut(ptr, len);
            let info = (*frame_buffer).framebuffers()[0].as_ptr();
            LockedLogger(Spinlock::new(Logger::new(frame, info)))
        }
    }
    /*pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        LockedLogger(Spinlock::new(Logger::new(framebuffer, info)))
    }*/

    /// Force-unlocks the logger to prevent a deadlock.
    ///
    /// This method is not memory safe and should be only used when absolutely necessary.
    pub unsafe fn force_unlock(&self) {
        unsafe { self.0.force_unlock() };
    }
}

/// Allows logging text to a pixel-based framebuffer.
pub struct Logger {
    framebuffer: &'static mut [u32],
    info: *mut LimineFramebuffer,
    x_pos: usize,
    y_pos: usize,
    screen_color: u32,
    text_color: u32
}

impl Logger {
    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'static mut [u32], info: *mut LimineFramebuffer) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            screen_color: 0,
            text_color: 0xFFFFFFFF
        };
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.y_pos += 8 + LINE_SPACING;
        self.carriage_return()
    }

    fn _add_vspace(&mut self, space: usize) {
        self.y_pos += space;
    }

    fn carriage_return(&mut self) {
        self.x_pos = 0;
    }

    /// Erases all text on the screen.
    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        (unsafe {&*self.info}).width as usize
    }

    fn height(&self) -> usize {
        (unsafe {&*self.info}).height as usize
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                if self.x_pos >= self.width() {
                    self.newline();
                }
                if self.y_pos >= (self.height() - 8) {
                    self.clear();
                }
                let rendered = font8x8::BASIC_FONTS
                    .get(c)
                    .expect("character not found in basic font");
                self.write_rendered_char(rendered);
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: [u8; 8]) {
        for (y, byte) in rendered_char.iter().enumerate() {
            for (x, bit) in (0..8).enumerate() {
                let color = if *byte & (1 << bit) == 0 { self.screen_color } else { self.text_color };
                self.write_pixel(self.x_pos + x, self.y_pos + y, color);
            }
        }
        self.x_pos += 8;
    }

    fn write_pixel(&mut self, x: usize, y: usize, color32: u32) {
        let pixel_offset = (y * (unsafe {&*self.info}).width as usize) + x;
        /*let color = match self.info.pixel_format {
            PixelFormat::RGB => [intensity, intensity, intensity / 2, 0],
            PixelFormat::BGR => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
        };*/
        let color = [color32];
        let bytes_per_pixel = ((unsafe {&*self.info}).bpp / 8) as usize;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    unsafe {
        LOGGER
            .get()
            .unwrap()
            .0.lock()
            .write_fmt(args)
            .expect("Printing to vga failed");
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::output::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}