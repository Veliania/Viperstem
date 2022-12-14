use font8x8::UnicodeFonts;
use limine::{LimineFramebufferRequest, LimineFramebuffer};
use spin::Mutex;
use x86_64::VirtAddr;

use crate::println;

pub struct Pos {
	pub y: usize,
	pub x: usize
}

pub struct Terminal;

impl Pos {
	pub fn new(x: usize, y: usize) -> Pos {
		Pos {
			y,
			x
		}
	}

	pub fn from_offset(offset: usize) -> Pos {
		let x = offset % unsafe {&FRAME_REQUEST}
			.get_response().get().expect("Failed to grab frame response from limine")
			.framebuffers().get(0).expect("Failed to grab frame response from limine")
			.width as usize;

		Pos {
			x,
			y: offset - x
		}
	}

	pub fn to_offset(&self) -> usize {
		self.x + (self.y * unsafe {&FRAME_REQUEST}.get_response().get().expect("Failed to grabe frame response from limine")
			.framebuffers().get(0).expect("Failed to grab frame response from limine").width as usize
		)
	}
}

pub static mut FRAME_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

static mut TERM: Terminal = Terminal;
static mut SCREEN: Mutex<Option<&LimineFramebuffer>> = Mutex::new(None);
pub static mut GLOB_POS: Pos = Pos {x: 0, y: 0};
static mut SCREEN_DAT: (u16, usize, usize) = (0, 0, 0);
static mut SCREEN_COLOR: u32 = 0x00;
static mut TEXT_COLOR: u32 = 0xFFFFFFFF;
pub static mut ADDRESS: Mutex<Option<VirtAddr>> = Mutex::new(None);

pub fn init() {
	unsafe {
		let frame_response = FRAME_REQUEST.get_response().get().expect("Failed to grab frame response from limine");
		
		let limine_frame_ptr = frame_response
			.framebuffers()
			.get(0)
			.expect("Failed to get framebuffer from frame response");

		SCREEN_DAT = (limine_frame_ptr.bpp / 32, limine_frame_ptr.size(), (limine_frame_ptr.height * limine_frame_ptr.width) as usize);
		
		let frame_count = frame_response.framebuffer_count;

		let frame = &*frame_response
			.framebuffers()
			.get(0)
			.expect("Failed to get framebuffer from frame response").as_ptr();
		
		*SCREEN.lock() = Some(frame);
		*ADDRESS.lock() = Some(VirtAddr::new_truncate(frame.address.as_ptr().unwrap() as u64));

		println!("Total frames: {}", frame_count);
	}
}

pub fn init_post_paging(addr: VirtAddr) {
	unsafe {
		*ADDRESS.lock() = Some(addr);
		GLOB_POS = Pos::new(0, 0);
	}
}

pub fn write_pix(offset: isize, data: u32) {
	unsafe {
		//let addr = SCREEN.lock().as_deref().expect("Failed to grab framebuffer").address.as_ptr().expect("Invalid address") as *mut u32;
		let addr = ADDRESS.lock().unwrap().as_mut_ptr() as *mut u32;

		*addr.offset(offset * SCREEN_DAT.0 as isize) = data;
	}
}

pub fn fill(data: u32) {
	unsafe {
		GLOB_POS = Pos {
			x: 0,
			y: 0
		};
		
		for i in 0..SCREEN_DAT.2 as isize {
			write_pix(i, data)
		}
	}
}

pub fn set_colors(back: u32, fore: u32) {
	unsafe {
		SCREEN_COLOR = back;
		TEXT_COLOR = fore;

		fill(SCREEN_COLOR);
	}
}

pub fn write_char(character: char) {
	const NULLCHAR: char = 0 as char;
	
	let rend_char = match character {
		NULLCHAR => [0; 8],
		_ => font8x8::BASIC_FONTS.get_font(character).expect("Character not found").byte_array()
	};
	
	unsafe {
		for (y, byte) in rend_char.iter().enumerate() {
			for (x, bit) in (0..8).enumerate() {
				let alpha = if *byte & (1 << bit) == 0 { SCREEN_COLOR } else { TEXT_COLOR };
				write_pix(Pos::to_offset(&Pos {x: GLOB_POS.x + x, y: GLOB_POS.y + y}) as isize, alpha);
			}
		}
	}
}

pub fn write_char_pos(character: char, position: &Pos) {
	const NULLCHAR: char = 0 as char;
	
	let rend_char = match character {
		NULLCHAR => [0; 8],
		_ => font8x8::BASIC_FONTS.get_font(character).expect("Character not found").byte_array()
	};
	
	unsafe {
		for (y, byte) in rend_char.iter().enumerate() {
			for (x, bit) in (0..8).enumerate() {
				let alpha = if *byte & (1 << bit) == 0 { SCREEN_COLOR } else { TEXT_COLOR };
				write_pix(Pos::to_offset(&Pos {x: position.x + x, y: position.y + y}) as isize, alpha);
			}
		}
	}
}

pub fn is_null_char(position: &Pos) -> bool {
	unsafe {
		//let addr = SCREEN.lock().as_deref().expect("Failed to grab framebuffer").address.as_ptr().expect("Invalid address") as *mut u32;
		let addr = ADDRESS.lock().unwrap().as_mut_ptr() as *mut u32;

		for x in 0..8 {
			for y in 0..8 {
				let newpos = Pos {
					x: position.x + x,
					y: position.y + y
				};
				if *addr.offset(newpos.to_offset() as isize) != 0 {
					return true;
				}
			}
		}

		return false;
	}
}

pub fn back(position: &Pos) {
	write_char_pos(0 as char, position);
}

use core::fmt;

impl fmt::Write for Terminal {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		unsafe {
			for character in s.chars() {
				match character {
					'\n' => {
						GLOB_POS.y += 8;
						GLOB_POS.x = 0;
					},
					_ => {
						write_char(character);
						GLOB_POS.x += 8;

						let width = *&FRAME_REQUEST
							.get_response().get().expect("Failed to grab frame response from limine")
							.framebuffers().get(0).expect("Failed to grab frame response from limine")
							.width as usize;

						if GLOB_POS.x > width {
							GLOB_POS.x = 0;
							GLOB_POS.y += 8;
						}
					}
				}
			}
		}
		Ok(())
	}
}
 
/*use core::fmt;

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
		unsafe {
			for character in s.chars() {
				match character {
					'\n' => {
						Y += 1;
						X += 0;
					}
					_ => {
						TEXT.lock().write_character(X, Y, ScreenCharacter::new(character as u8, COLOR));
						X += 1;

						if X > 80 {
							Y += 1;
							X = 0;
						}
						TEXT.lock().set_cursor_position(X, Y);
					}
				}
			}
		}

		Ok(())
    }
}

pub fn test() {
	use crate::println;

	unsafe {
		X = 0;
		Y = 0;
	}

	for y in 0..80 {
		unsafe {
			X = 0;
		}
		println!("{}", y);
	}

	unsafe {
		TEXT.lock().clear_screen();
		TEXT.lock().write_character(0, 0, ScreenCharacter::new(b'A', COLOR));
		TEXT.lock().write_character(1, 0, ScreenCharacter::new(b'A', COLOR));
		for i in 0..80 {
			TEXT.lock().write_character(i * 40, 0, ScreenCharacter::new((b'A') + (i / 40) as u8, COLOR))
		}
	}
}
*/

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    //use x86_64::instructions::interrupts;

    //interrupts::without_interrupts(|| {
		unsafe {
			TERM
				.write_fmt(args)
				.expect("Printing to vga failed");
		}
    //});
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