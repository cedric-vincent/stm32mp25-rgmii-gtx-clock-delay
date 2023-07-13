mod ethtool;

use crate::error::Error;
use crate::clock_delay;

use byte_unit::Byte;
use std::time::{Instant, Duration};

pub(crate) fn perform(device: &str, url: &str, size_threshold: Byte, time_threshold: u64) -> Result<(), Error> {
	let reversed_valid_values = clock_delay::VALID_VALUES.iter().cloned().rev().collect::<Vec<_>>();

	println!("Using URL {url}");

	println!("Pass 1/2");
	let results1 = perform_single_pass(device, url, size_threshold, time_threshold, &clock_delay::VALID_VALUES);

	println!("Pass 2/2");
	let results2 = perform_single_pass(device, url, size_threshold, time_threshold, &reversed_valid_values);

/*
	// TODO: consider duration and neighbors too!
	results.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

	print!("RGMII GTX clock delay sorted from best to worst: ");
	for (clock_delay, _, _) in &results {
		print!("{clock_delay}, ");
	}
	println!("");
*/
	Ok(())
}

fn perform_single_pass(device: &str, url: &str, size_threshold: Byte, time_threshold: u64, delays: &[f32]) -> Result<Vec<(f32, f64, Duration)>, Error> {
	let mut results = vec![];

	for clock_delay in delays.iter() {
		use std::io::Write;

		let clock_delay = *clock_delay;

		clock_delay::access(device, Some(clock_delay), false)?;

		let message = format!("Benchmarking with RGMII GTX clock delay = {clock_delay:.2} nanoseconds... ");
		let _ = std::io::stdout().write(message.as_bytes());
		let _ = std::io::stdout().flush();

		let start = get_info(device)?;

		let status = download(url, size_threshold, time_threshold);
		if let Err(error) = &status {
			if let Error::Download(error) = error {
				if error.is_operation_timedout() {
					println!("{error}");
					continue;
				}
			}
		}
		status?;

		let end = get_info(device)?;

		let mmc_rx_crc_error = end.mmc_rx_crc_error - start.mmc_rx_crc_error;
		let rx_pkt_n         = end.rx_pkt_n         - start.rx_pkt_n;
		let percent          = (100 * mmc_rx_crc_error) as f64 / rx_pkt_n as f64;
		let duration         = end.instant - start.instant;

		println!("It took {:.2}s; CRC error rate is {percent:.2}% ({mmc_rx_crc_error}/{rx_pkt_n})", duration.as_secs_f32());

		results.push((clock_delay, percent, duration));
	}

	Ok(results)
}

fn download(url: &str, size_threshold: Byte, time_threshold: u64) -> Result<(), Error> {
	use curl::easy as curl;

	let mut handle = curl::Easy::new();

	handle.url(url)?;
	handle.fail_on_error(true)?;

	// Abort if transfer speed is < size_threshold bytes / time_threshold seconds.
	let time_threshold = Duration::from_secs(time_threshold);

	handle.low_speed_limit(size_threshold.get_bytes() as u32)?;
	handle.low_speed_time(time_threshold)?;
	handle.connect_timeout(time_threshold)?;

	let curl_result = {
		let mut transfer = handle.transfer();

		transfer.write_function(|data| {
			Ok(data.len())
		})?;

		transfer.perform()
	};

	curl_result?;

	Ok(())
}

fn get_info(device: &str) -> Result<Info, Error> {
	// TODO: handle all these .unwrap()
	let nic_stats = ethtool::get_nic_stats(device).unwrap();

	Ok(Info {
		mmc_rx_crc_error: *nic_stats.get("mmc_rx_crc_error").unwrap(),
		rx_pkt_n:         *nic_stats.get("rx_pkt_n").unwrap(),
		instant:          Instant::now(),
	})
}

struct Info {
	mmc_rx_crc_error: u64,
	rx_pkt_n:         u64,
	instant:          Instant,
}
