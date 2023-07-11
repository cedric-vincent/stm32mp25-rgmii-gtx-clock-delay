use byte_unit::{Byte, AdjustedByte};
use std::time::{Instant, Duration};

pub struct StatusBar {
	current_time:       Instant,
	total_time:         Instant,
	current_transfered: u64,
	total_transfered:   u64,
	total_size:         u64,
	max_speed:          AdjustedByte,
	min_speed:          AdjustedByte,
	max_message_len:    usize,
	print_every:        Duration,
}

impl StatusBar {
	pub fn new (total_size: u64, print_every: Duration) -> Self {
		Self {
			current_time:       Instant::now(),
			total_time:         Instant::now(),
			current_transfered: 0_u64,
			total_transfered:   0_u64,
			total_size,
			max_speed:          Byte::from_bytes(u128::MIN).get_appropriate_unit(false),
			min_speed:          Byte::from_bytes(u128::MAX).get_appropriate_unit(false),
			max_message_len:    0,
			print_every,
		}
	}

	pub fn update (&mut self, size: u64) {
		use std::io::Write;

		self.current_transfered += size as u64;

		if self.current_time.elapsed() < self.print_every {
			return;
		}

		self.total_transfered += self.current_transfered;

		let current_speed = compute_speed(self.current_transfered, self.current_time);
		let mean_speed    = compute_speed(self.total_transfered, self.total_time);

		self.min_speed = std::cmp::min(current_speed, self.min_speed);
		self.max_speed = std::cmp::max(current_speed, self.max_speed);

		let percent = if self.total_size > 0 {
			format!("{}%  ", 100 * self.total_transfered / self.total_size)
		} else {
			String::new()
		};

		let message = format!("\r{}current speed = {}/s  mean = {}/s  min. = {}/s  max. = {}/s",
		                      percent, current_speed, mean_speed, self.min_speed, self.max_speed);

		self.max_message_len = std::cmp::max(self.max_message_len, message.len());

		let message = format!("{:<width$}", message, width = self.max_message_len);

		let _ = std::io::stdout().write(message.as_bytes());
		let _ = std::io::stdout().flush();

		self.current_time       = Instant::now();
		self.current_transfered = 0;

		fn compute_speed (amount: u64, time: Instant) -> AdjustedByte {
			let time_elapsed = time.elapsed().as_millis();

			let bytes = if time_elapsed  == 0 {
				0
			} else {
				1000 * amount as u128 / time_elapsed
			};

			Byte::from_bytes(bytes).get_appropriate_unit(false)
		}
	}

	pub fn end (&mut self) {
		if self.print_every == Duration::MAX {
			return;
		}

		let size = if self.total_size > 0 {
			self.total_size - (self.total_transfered + self.current_transfered)
		} else {
			0
		};

		self.print_every = Duration::ZERO;
		self.update(size);
		println!();
	}
}
