use std::path::{Path, PathBuf};
use std::io::Read;

use crate::clock_delay::Gpio;
use crate::error;

pub(crate) fn get_name (device: &str) -> Result<String, error::GetDtName> {
	use std::io::BufRead;

	let path   = format!("/sys/class/net/{device}/device/uevent");
	let handle = std::fs::File::open(path)?;
	let reader = std::io::BufReader::new(handle);

	for line in reader.lines().flatten() {
		let mut tokens = line.split('=');

		if tokens.next() == Some("OF_NAME") {
			return match tokens.next() {
				Some(token) => Ok(String::from(token)),
				None        => Err(std::io::Error::from(std::io::ErrorKind::NotFound))?,
			}
		}
	}

	Err(std::io::Error::from(std::io::ErrorKind::NotFound))?
}

pub(crate) fn find_nodes(gpio: &Gpio) -> Vec<String> {
	let mut paths = Vec::new();

	let base = "/sys/firmware/devicetree/base";

	find_paths(&format!("{base}/soc/{}", gpio.pinctrl), &gpio, &mut paths);

	paths.iter().map(|path| format!("/{}", path.strip_prefix(base).unwrap().display())).collect()
}

fn find_paths<P: AsRef<Path>>(current_dir: P, gpio: &Gpio, result: &mut Vec<PathBuf>) {
	let current_dir  = current_dir.as_ref();
	let current_dir_ = format!("{}", current_dir.display());

	if ! current_dir.is_dir() {
		log::warn!("{current_dir_} is not a directory or does not exist");
		return;
	}

	let read_dir = match std::fs::read_dir(current_dir) {
		Err(error)   => { log::warn!("{error} while opening {current_dir_}"); return }
		Ok(read_dir) => { read_dir }
	};

	for entry in read_dir {
		let entry = match entry {
			Err(error) => { log::warn!("{error} while reading {current_dir_}"); continue }
			Ok(entry)  => { entry }
		};

		let file_type = match entry.file_type() {
			Err(error)    => { log::warn!("{error} while reading {current_dir_}"); continue }
			Ok(file_type) => { file_type }
		};

		if file_type.is_dir() {
			find_paths(&entry.path(), gpio, result);
		} else if file_type.is_file() {
			if entry.file_name() != "pinmux" {
				continue;
			}

			let handle = match std::fs::File::open(entry.path()) {
				Err(error) => { log::warn!("{error} while opening {:?}", entry.path()); continue }
				Ok(handle) => { handle }
			};

			let mut reader = std::io::BufReader::new(handle);
			let mut buffer = [0u8; 4];

			loop {
				match reader.read(&mut buffer) {
					Err(error) => { log::warn!("{error} while reading {:?}", entry.path()); continue }
					Ok(4)      => {
						let value  = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
						let pinmux = PinMux::from(value);

						if pinmux.bank == gpio.bank as u8 - b'A' && pinmux.line == gpio.line {
							let mut path = entry.path();
							path.pop();
							result.push(path);
						}
					}
					Ok(0) => { break }
					Ok(_) => { log::warn!("Unexpected end-of-file while reading {:?}", entry.path()); continue }
				}
			}
		}
	}
}

#[derive(Debug)]
struct PinMux {
	bank:  u8,
	line:  u8,
	_mode: u8,
}

impl From<u32> for PinMux {
	fn from(value: u32) -> Self {
		PinMux {
			_mode: (value & 0xFF)           as u8,
			line:  ((value & 0xF00) >> 8)   as u8,
			bank:  ((value & 0xF000) >> 12) as u8,
		}
	}
}

#[test]
fn from_u32_for_pinmux () {
	let pinmux = PinMux::from(0x0000580b);

	assert_eq!(pinmux.bank,  5);
	assert_eq!(pinmux.line,  8);
	assert_eq!(pinmux._mode, 0xb);
}
