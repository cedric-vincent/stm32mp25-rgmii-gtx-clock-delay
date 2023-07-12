mod status_bar;

use crate::error::Error;
use crate::clock_delay;

use status_bar::StatusBar;
use byte_unit::Byte;
use std::time::Duration;

pub(crate) fn perform(device: &str, url: &str, first_clock_delay: f32, last_clock_delay: f32, size_threshold: Byte, time_threshold: u64) -> Result<(), Error> {
	log::info!("using URL: {url}");

	for value in clock_delay::VALID_VALUES.iter() {
		let value = *value;

		if value < first_clock_delay || value > last_clock_delay {
			continue;
		}

		log::info!("setting RGMII GTX clock delay to {value} nanoseconds");
		clock_delay::access(device, Some(value), false)?;

		let status = download(url, first_clock_delay, last_clock_delay, size_threshold, time_threshold);
		if let Err(error) = &status {
			if let Error::Download(error) = error {
				if error.is_operation_timedout() {
					eprintln!("");
					log::warn!("{error}");
					continue;
				}
			}
		}
		status?;
	}

	Ok(())
}

fn download(url: &str, _first_clock_delay: f32, _last_clock_delay: f32, size_threshold: Byte, time_threshold: u64) -> Result<(), Error> {
	use curl::easy as curl;

	let mut handle = curl::Easy::new();

	handle.url(url)?;
	handle.fail_on_error(true)?;

	// Abort if transfer speed is < size_threshold bytes / time_threshold seconds.
	handle.low_speed_limit(size_threshold.get_bytes() as u32)?;
	handle.low_speed_time(Duration::from_secs(time_threshold))?;

	let total_size = match (handle.perform(), handle.content_length_download()) {
		(Ok(_), Ok(length)) => length as u64,
		_                   => 0,
	};

	let mut transfer_status = StatusBar::new(total_size, Duration::from_secs(1));

	let curl_result = {
		let mut transfer = handle.transfer();

		transfer.write_function(|data| {
			transfer_status.update(data.len() as u64);
			Ok(data.len())
		})?;

		transfer.perform()
	};

	curl_result?;
	transfer_status.end();

	Ok(())
}