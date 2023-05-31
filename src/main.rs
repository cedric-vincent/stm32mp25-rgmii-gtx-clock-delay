fn main () -> Result<(), std::io::Error> {
	let dev_name = "eth1";
	let dt_name  = find_dt_name(dev_name)?.unwrap();
	let gpio     = find_gpio(&dt_name)?.unwrap();
	let address  = find_address(&gpio)?.unwrap();

	// TODO: log::info instead of println
	println!("device named \"{dev_name}\" is known as \"{dt_name}\" in device-tree");
	println!("↳ its RGMII GTX clock is connected to GPIO {gpio}");
	println!("  ↳ its delay can be accessed at address {address:#x} in /dev/mem");

	Ok(())
}

#[derive(Debug)]
struct Gpio {
	bank: char,
	line: u8,
}

impl std::fmt::Display for Gpio {
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(formatter, "{}{}", self.bank, self.line)
	}
}

fn find_address (gpio: &Gpio) -> Result<Option<usize>, std::io::Error> {
	let path = format!("/sys/firmware/devicetree/base/__symbols__/gpio{}", gpio.bank.to_lowercase());
	let path = std::fs::read_to_string(path)?;
	let path = path.trim_end_matches('\0');

	match path.split('@').last() {
		None          => Ok(None),
		Some(address) => {
			match usize::from_str_radix(address, 16) {
				Ok(address) => Ok(Some(address + 0x40)), // TODO: + gpio.line / 2
				Err(_)      => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)) // TODO: user friendly error
			}
		}
	}
}

fn find_gpio (dt_name: &str) -> Result<Option<Gpio>, std::io::Error> {
	use std::io::BufRead;

	let entries = std::fs::read_dir("/sys/kernel/debug/pinctrl/")?; // TODO: user friendly error
	for entry in entries {
		let entry = entry?; // TODO: user friendly error

		let file_name = entry.file_name();
		let file_name = file_name.to_string_lossy();

		let mut tokens = file_name.split('@');

		if tokens.next() != Some("soc:pinctrl") {
			continue;
		}

		let mut path = entry.path();
		path.push("pinconf-pins");

		let handle = std::fs::File::open(path)?; // TODO: user friendly error
		let reader = std::io::BufReader::new(handle);
		let needle = format!("{}_RGMII_GTX_CLK", dt_name.to_uppercase());

		// TODO: user friendly error
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
				continue; // TODO: user friendly error
			}

			return Ok(Some(Gpio {
				bank: bank.unwrap(),
				line: line.unwrap(),
			}));
		}
	}

	Ok(None)
}

fn find_dt_name (dev_name: &str) -> Result<Option<String>, std::io::Error> {
	use std::io::BufRead;

	let path   = format!("/sys/class/net/{dev_name}/device/uevent");
	let handle = std::fs::File::open(path)?; // TODO: user friendly error
	let reader = std::io::BufReader::new(handle);

	for line in reader.lines().flatten() {
		let mut tokens = line.split('=');

		if tokens.next() == Some("OF_NAME") {
			return Ok(tokens.next().map(String::from));
		}
	}

	Ok(None)
}
