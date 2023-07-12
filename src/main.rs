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
	        .verbosity(options.verbose as usize + 2)
	        .color(stderrlog::ColorChoice::Never)
	        .init();

	let status = match main2(options) {
		Ok(())     => { log::info!("Success!"); 0 }
		Err(error) => { log::error!("{error}"); 1 }
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
			size_threshold:    e,
			time_threshold:    f }       => benchmark::perform(&a, &b, c, d, e, f)?,
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

		/// Benchmark by fetching content from this URL (recommended size > 100 MiB)
		#[clap(short, long, default_value = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.4.3.tar.xz")]
		url: String,

		/// First benchmarked value (in ns)
		#[clap(short, long, default_value = "0", value_parser = clock_delay::parser)]
		first_clock_delay: f32,

		/// Last benchmarked value (in ns)
		#[clap(short, long, default_value = "3.25", value_parser = clock_delay::parser)]
		last_clock_delay: f32,

		/// Skip if throughput is less than SIZE_THRESHOLD bytes/second during TIME_THRESHOLD seconds
		#[clap(short, long, default_value = "100 kiB", value_parser = size_threshold_parser)]
		size_threshold: Byte,

		/// Skip if throughput is less than SIZE_THRESHOLD bytes/second during TIME_THRESHOLD seconds
		#[clap(short, long, default_value = "10")]
		time_threshold: u64,
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

fn size_threshold_parser (value: &str) -> Result<Byte, String> {
	Byte::from_str(value).map_err(|error| format!("not a valid size in bytes ({error})"))
}
