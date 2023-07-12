mod error;
mod clock_delay;
mod benchmark;

use error::Error;
use clap::{Parser, Subcommand};
use byte_unit::Byte;

#[macro_use]
extern crate lazy_static;

fn main () {
	let options = Options::parse();

	let _ = stderrlog::new()
	        .verbosity(options.verbose as usize)
	        .color(stderrlog::ColorChoice::Never)
	        .init();

	let status = match main2(options) {
		Ok(())     => { 0 }
		Err(error) => { eprintln!("Error: {error}"); 1 }
	};

	std::process::exit(status);
}

fn main2 (options: Options) -> Result<(), Error> {
	match options.command {
		Command::Benchmark {
			device:            a,
			url:               b,
			first_clock_delay: c,
			last_clock_delay:  d,
			speed_low_limit:   e,
			timeout:           f }       => benchmark::perform(&a, &b, c, d, e, f)?,
		Command::Set { device, clock_delay } => clock_delay::access(&device, Some(clock_delay), true)?,
		Command::Get { device }              => clock_delay::access(&device, None, true)?,
	}

	Ok(())
}

#[derive(Parser)]
struct Options {
	/// Increase verbosity level (once = debug, twice = trace)
	#[clap(short, long, action = clap::ArgAction::Count)]
	verbose: u8,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
#[clap(author, version, about = "Calibrate STM32MP25 RGMII TX clock delay")]
enum Command {
	Benchmark {
		/// Device name
		#[clap(short, long)]
		device: String,

		/// Benchmark by fetching data from this URL (recommended size > 100 MiB)
		#[clap(short, long, default_value = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.4.3.tar.xz")]
		url: String,

		/// First benchmarked value (in ns)
		#[clap(short, long, default_value = "0", value_parser = clock_delay::parser)]
		first_clock_delay: f32,

		/// Last benchmarked value (in ns)
		#[clap(short, long, default_value = "3.25", value_parser = clock_delay::parser)]
		last_clock_delay: f32,

		/// Skip if transfer rate is below SPEED_LOW_LIMIT bytes/second during more than TIMEOUT seconds
		#[clap(short, long, default_value = "100 kiB", value_parser = speed_low_limit_parser)]
		speed_low_limit: Byte,

		/// Timemout for SPEED_LOW_LIMIT and for the connection phase.
		#[clap(short, long, default_value = "5")]
		timeout: u64,
	},

	Set {
		/// Device name
		#[clap(short, long)]
		device: String,

		/// RGMII GTX clock delay (in ns)
		#[clap(short, long, value_parser = clock_delay::parser)]
		clock_delay: f32,
	},

	Get {
		/// Device name
		#[clap(short, long)]
		device: String,
	}
}

fn speed_low_limit_parser (value: &str) -> Result<Byte, String> {
	Byte::from_str(value).map_err(|error| format!("not a valid size in bytes ({error})"))
}
