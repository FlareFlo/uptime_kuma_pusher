use std::{
	io,
	thread::sleep,
	time::{Duration, Instant},
};

use url::Url;

pub struct UptimePusher {
	url:             Url,
	heartbeat_delay: Duration,
	silent:          bool,
}

impl UptimePusher {
	/// Constructs new pusher, panics if any configuration is invalid.
	/// If anyone caught this error it would make the concept of an uptime pusher redundant.
	pub fn new(url: &str, silent: bool) -> Self {
		Self {
			url: Url::parse(url).unwrap(),
			// Uptime Kuma expects a heartbeat every 60 seconds, 5 seconds of ping/jitter is plentiful
			heartbeat_delay: Duration::from_secs(55),
			silent,
		}
	}

	/// Spawns thread that polls server regularly with UP status
	pub fn spawn_background(self) {
		std::thread::spawn(move || loop {
			let e = self.push_ok();
			if e.is_err() && self.silent {
				let _ = dbg!(e);
			}

			let now = Instant::now();
			if let Some(dur) = (now + self.heartbeat_delay).checked_duration_since(now) {
				sleep(dur)
			}
		});
	}

	/// Pushes message with arguments to server
	pub fn try_push_status_and_msg(&self, status_ok: bool, msg: &str) -> io::Result<()> {
		let mut url = self.url.clone();
		url.query_pairs_mut()
			.append_pair("status", if status_ok { "up" } else { "down" })
			.append_pair("msg", msg);

		let res = ureq::get(url.as_str())
			.timeout(Duration::from_secs(10))
			.call();
		if let Err(res) = res {
			if !self.silent {
				println!("Failed to push uptime-kuma: {}", res);
			}
		}
		Ok(())
	}

	/// Shorthand for pushing UP
	pub fn push_ok(&self) -> io::Result<()> {
		self.try_push_status_and_msg(true, "")
	}
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use crate::UptimePusher;

	#[test]
	fn simple_push() {
		let p = UptimePusher::new("foobar", false);
		p.spawn_background();
		sleep(Duration::from_secs(100));
	}
}
