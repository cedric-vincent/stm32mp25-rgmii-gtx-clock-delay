mod error;

fn main () {
	let _ = stderrlog::new()
	        .verbosity(2) // + options.verbose
	        .color(stderrlog::ColorChoice::Never)
	        .init();

	match main2() {
		Ok(())     => { std::process::exit(0) }
		Err(error) => { log::error!("{error}"); std::process::exit(1) }
	}
}

fn main2 () -> Result<(), error::Error> {
	let dev_name = "eth1";

	let dt_name = get_dt_name(dev_name)?;
	let gpio    = get_gpio(&dt_name)?;
	let address = get_address(&gpio)?;

	log::info!("device named \"{dev_name}\" is known as \"{dt_name}\" in device-tree");
	log::info!("↳ its RGMII GTX clock is connected to GPIO {gpio}");
	log::info!("  ↳ its delay can be accessed at address {address} in /dev/mem");

	Ok(())
}

fn get_dt_name (dev_name: &str) -> Result<String, error::GetDtName> { //std::io::Error> {
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
