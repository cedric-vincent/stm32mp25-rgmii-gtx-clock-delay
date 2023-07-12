mod ethtool;

use crate::error::Error;
use crate::clock_delay;

use byte_unit::Byte;
use std::time::Duration;

pub(crate) fn perform(device: &str, url: &str, size_threshold: Byte, time_threshold: u64) -> Result<(), Error> {
	use std::io::Write;

	let mut results = vec![];

	println!("Using URL {url}");

	// TODO: in one direction then in the opposite
	for clock_delay in clock_delay::VALID_VALUES.iter() {
		let clock_delay = *clock_delay;

		clock_delay::access(device, Some(clock_delay), false)?;

		let start = ethtool::get_nic_stats(device).unwrap();

		let message = format!("Benchmarking with RGMII GTX clock delay = {clock_delay:.2} nanoseconds... ");
		let _ = std::io::stdout().write(message.as_bytes());
		let _ = std::io::stdout().flush();

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

		// TODO: handle all these .unwrap()
		let end = ethtool::get_nic_stats(device).unwrap();

		let mmc_rx_crc_error = end.get("mmc_rx_crc_error").unwrap() - start.get("mmc_rx_crc_error").unwrap();
		let rx_pkt_n         = end.get("rx_pkt_n").unwrap() - start.get("rx_pkt_n").unwrap();
		let percent          = (100 * mmc_rx_crc_error) as f64 / rx_pkt_n as f64;

		println!("CRC errors per packet: {percent:.2}% ({mmc_rx_crc_error}/{rx_pkt_n})");

		results.push((clock_delay, percent));
	}

	results.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

	print!("RGMII GTX clock delay sorted from best to worst: ");
	for (clock_delay, _) in &results {
		print!("{clock_delay}, ");
	}
	println!("");

	Ok(())
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
