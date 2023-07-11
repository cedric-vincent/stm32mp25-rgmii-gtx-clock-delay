mod error;
use error::Error;
use clap::{Args, Parser, Subcommand};

fn main () {
	let _ = stderrlog::new()
	        .color(stderrlog::ColorChoice::Never)
	        .init();

	let options = Options::parse();

	match main2(options) {
		Ok(())     => { std::process::exit(0) }
		Err(error) => { log::error!("{error}"); std::process::exit(1) }
	}
}

fn main2 (options: Options) -> Result<(), Error> {
	match options.command {
		Command::Benchmark { .. }     => { todo!() }

		Command::Set       { .. }     => { todo!() }

		Command::Get       { device } => {
			let dt_name = get_dt_name(&device)?;
			log::info!("device named \"{device}\" is known as \"{dt_name}\" in device-tree");

			let gpio = get_gpio(&dt_name)?;
			log::info!("↳ its RGMII GTX clock is connected to GPIO {gpio}");

			let address = get_address(&gpio)?;
			log::info!("  ↳ its delay can be accessed at address {address} in /dev/mem");

			let value = get_value(&address)?;
			log::info!("    ↳ its value is {value:#x} ({} nanoseconds)", convert_to_ns(value)?);
		}
	}

	Ok(())
}

fn get_dt_name (dev_name: &str) -> Result<String, error::GetDtName> {
	use std::io::BufRead;

	let path   = format!("/sys/class/net/{dev_name}/device/uevent");
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

fn get_gpio (dt_name: &str) -> Result<Gpio, error::GetGpio> {
	use std::io::BufRead;

	let entries = std::fs::read_dir("/sys/kernel/debug/pinctrl/")?;
	for entry in entries {
		let entry = entry?;

		let file_name = entry.file_name();
		let file_name = file_name.to_string_lossy();

		let mut tokens = file_name.split('@');

		if tokens.next() != Some("soc:pinctrl") {
			continue;
		}

		let mut path = entry.path();
		path.push("pinconf-pins");

		let handle = std::fs::File::open(path)?;
		let reader = std::io::BufReader::new(handle);
		let needle = format!("{}_RGMII_GTX_CLK", dt_name.to_uppercase());

		let error = || { std::io::Error::from(std::io::ErrorKind::InvalidData) };

		for line in reader.lines().flatten() {
			if ! line.contains(&needle) {
				continue;
			}

			let mut tokens = line.split('(');
			let     tokens = tokens.nth(1).ok_or_else(error)?;
			let mut tokens = tokens.split(')');
			let     token  = tokens.nth(0).ok_or_else(error)?;
			let mut tokens = token.chars();

			let magic = tokens.next();
			let bank  = tokens.next();
			let line  = tokens.as_str().parse::<u8>().ok();

			if magic != Some('P') || bank.is_none() || line.is_none() {
				continue;
			}

			return Ok(Gpio {
				bank: bank.unwrap(),
				line: line.unwrap(),
			});
		}
	}

	Err(std::io::Error::from(std::io::ErrorKind::NotFound))?
}

fn get_address (gpio: &Gpio) -> Result<Address, error::GetAddress> {
	let path = format!("/sys/firmware/devicetree/base/__symbols__/gpio{}", gpio.bank.to_lowercase());
	let path = std::fs::read_to_string(path)?;
	let path = path.trim_end_matches('\0');

	match path.split('@').last() {
		None          => Err(std::io::Error::from(std::io::ErrorKind::NotFound))?,
		Some(address) => {
			match usize::from_str_radix(address, 16) {
				Ok(address) => Ok(Address { base: address + 0x40, offset: gpio.line * 4 }),
				Err(_)      => Err(std::io::Error::from(std::io::ErrorKind::InvalidData))?,
			}
		}
	}
}

fn get_value (address: &Address) -> Result<u32, error::GetValue> {
	use nix::unistd::{sysconf, SysconfVar};
	use nix::sys::mman::{mmap, ProtFlags, MapFlags};
	use std::os::unix::io::AsRawFd;

	let handle = std::fs::File::open("/dev/mem")?;

	let page_size   = sysconf(SysconfVar::PAGE_SIZE)?.unwrap_or(4096) as usize;
	let length      = std::num::NonZeroUsize::new(page_size).unwrap();
	let page_base   = (address.base & !(page_size - 1)) as libc::off_t;
	let page_offset = address.base & (page_size - 1);

	let value = unsafe {
		*mmap(None, length, ProtFlags::PROT_READ, MapFlags::MAP_SHARED, handle.as_raw_fd(), page_base)?
		.add(page_offset)
		.cast::<u32>()
	};

	Ok((value >> address.offset) & 0xF)
}

fn convert_to_ns(value: u32) -> Result<f32, Error> {
	match value {
		0          => Ok(0.0),
		1          => Ok(0.3),
		x @ 2..=12 => Ok(0.25 * x as f32),
		13..=16    => Ok(3.25),
		_          => Err(Error::OutOfRangeDelay)
	}
}

#[derive(Parser)]
struct Options {
	#[clap(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
#[clap(author, version, about = "Calibrate STM32MP25 RGMII TX clock delay")]
enum Command {
	Benchmark {
		/// Device name
		#[clap(short = 'D', long)]
		device: String,

		#[command(flatten)]
		urls: Urls,

		/// First benchmarked value (in ns)
		#[clap(short, long, default_value = "0", value_parser = clock_delay_parser)]
		first_clock_delay: f32,

		/// Last benchmarked value (in ns)
		#[clap(short, long, default_value = "3.25", value_parser = clock_delay_parser)]
		last_clock_delay: f32,

		/// Skip if throughput is less than SIZE_THRESHOLD/TIME_THRESHOLD
		#[clap(short, long, default_value = "5 MiB")]
		size_threshold: String,

		/// Skip if throughput is less than SIZE_THRESHOLD/TIME_THRESHOLD
		#[clap(short, long, default_value = "5")]
		time_threshold: usize,
	},

	Set {
		/// Device name
		#[clap(short = 'D', long)]
		device: String,

		/// TODO
		clock_delays: u8,
	},

	Get {
		/// Device name
		#[clap(short = 'D', long)]
		device: String,
	}
}

#[derive(Args)]
#[group(required = true, multiple = true)]
struct Urls {
	/// Benchmark by downloading (and discarding) content from URL
	#[clap(short, long)]
	download_from: Option<String>,

	/// Benchmark by uploading (random) content to URL
	#[clap(short, long)]
	upload_to: Option<String>,
}

fn clock_delay_parser (value: &str) -> Result<f32, String> {
	match value.parse::<f32>() {
		Err(_)    => Err(format!("not a floating point value")),
		Ok(value) => {
			let mut valid_values = vec![0 as f32, 0.3];
			valid_values.append(&mut (2..=13).map(|x| x as f32 * 0.25).collect());

			if valid_values.contains(&value) {
				Ok(value)
			} else {
				Err(format!("must be one of {:?}", valid_values))
			}
		}
	}
}

#[derive(Debug)]
struct Gpio {
	bank: char,
	line: u8,
}

#[derive(Debug)]
struct Address {
	base:   usize,
	offset: u8,
}

impl std::fmt::Display for Gpio {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(formatter, "{}{}", self.bank, self.line)
	}
}

impl std::fmt::Display for Address {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(formatter, "{:#x} (bits {}-{})", self.base, self.offset, self.offset + 4)
	}
}
