mod status_bar;

use crate::error::Error;
use status_bar::StatusBar;

pub(crate) fn perform(_device: &str, url: &str, _first_clock_delay: f32, _last_clock_delay: f32, _size_threshold: &str, _time_threshold: usize) -> Result<(), Error> {
	use curl::easy as curl;
	use std::time::Duration;

	let mut handle = curl::Easy::new();

	handle.url(url)?;
	handle.fail_on_error(true)?;

	// Abort if transfer speed is < 1 b/s during 30 seconds.
	handle.low_speed_limit(1)?;
	handle.low_speed_time(Duration::from_secs(30))?;

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