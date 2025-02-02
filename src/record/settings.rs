use crate::args::matches::ArgMatches;
use crate::args::parser::ArgParser;
use crate::image::geometry::Geometry;
use crate::image::padding::Padding;
use crate::util::command::Command;

/* Time related recording settings */
#[derive(Clone, Copy, Debug)]
pub struct RecordTime {
	pub duration: Option<f64>,
	pub countdown: u64,
	pub timeout: u64,
	pub interval: u64,
}

/* Default initialization values for RecordTime */
impl Default for RecordTime {
	fn default() -> Self {
		Self {
			duration: None,
			countdown: 3,
			timeout: 300,
			interval: 10,
		}
	}
}

impl RecordTime {
	/**
	 * Create a new RecordTime object.
	 *
	 * @param  duration (Option)
	 * @param  countdown
	 * @param  timeout
	 * @param  interval
	 * @return RecordTime
	 */
	pub fn new(
		duration: Option<f64>,
		countdown: u64,
		timeout: u64,
		interval: u64,
	) -> Self {
		Self {
			duration,
			countdown,
			timeout,
			interval,
		}
	}

	/**
	 * Create a RecordTime object from an argument parser.
	 *
	 * @param  parser
	 * @return RecordTime
	 */
	fn from_parser(parser: &ArgParser<'_>) -> Self {
		RecordTime::new(
			match parser.parse("duration", 0.0) {
				duration if duration > 0.0 => Some(duration),
				_ => Self::default().duration,
			},
			parser.parse("countdown", Self::default().countdown),
			parser.parse("timeout", Self::default().timeout),
			parser.parse("interval", Self::default().interval),
		)
	}
}

/* Flag values of recording */
#[derive(Clone, Copy, Debug)]
pub struct RecordFlag {
	pub alpha: bool,
	pub action_keys: Option<&'static str>,
	pub cancel_keys: Option<&'static str>,
	pub font: Option<&'static str>,
	pub monitor: Option<usize>,
	pub select: bool,
	pub mouse: bool,
}

/* Default initialization values for RecordFlag */
impl Default for RecordFlag {
	fn default() -> Self {
		Self {
			alpha: false,
			action_keys: Some(""),
			cancel_keys: Some(""),
			font: None,
			monitor: None,
			select: true,
			mouse: false,
		}
	}
}

impl RecordFlag {
	/**
	 * Create a new RecordFlag object.
	 *
	 * @param  alpha
	 * @param  action_keys (Option)
	 * @param  cancel_keys (Option)
	 * @param  font
	 * @param  monitor (Option)
	 * @param  select
	 * @param  mouse
	 * @return RecordFlag
	 */
	pub fn new(
		alpha: bool,
		action_keys: Option<&'static str>,
		cancel_keys: Option<&'static str>,
		font: &str,
		monitor: Option<usize>,
		select: bool,
		mouse: bool,
	) -> Self {
		Self {
			alpha,
			action_keys,
			cancel_keys,
			font: if font.is_empty() {
				None
			} else {
				Some(Box::leak(font.to_string().into_boxed_str()))
			},
			monitor,
			select,
			mouse,
		}
	}
}

/* Window to record, with geometric properties  */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecordWindow {
	Focus(Option<Geometry>, bool),
	Root(Option<Geometry>),
}

impl RecordWindow {
	/**
	 * Create a RecordWindow enum from parsed arguments.
	 *
	 * @param  matches
	 * @return RecordWindow
	 */
	fn from_args(matches: &ArgMatches<'_>) -> Self {
		let size =
			if matches.occurrences_of("size") != 0 || matches.is_present("select") {
				Some(Geometry::parse(
					matches
						.value_of("size")
						.unwrap_or_default()
						.split('+')
						.collect::<Vec<&str>>()[0],
				))
			} else {
				None
			};
		if matches.is_present("focus") && !matches.is_present("monitor") {
			Self::Focus(size, matches.is_present("parent"))
		} else if matches.is_present("root") || matches.is_present("monitor") {
			Self::Root(size)
		} else {
			Self::Focus(Some(size.unwrap_or_default()), matches.is_present("parent"))
		}
	}
}

/* Recording and window settings */
#[derive(Clone, Copy, Debug)]
pub struct RecordSettings {
	pub command: Option<&'static str>,
	pub color: u64,
	pub border: Option<u32>,
	pub padding: Padding,
	pub time: RecordTime,
	pub flag: RecordFlag,
	pub window: RecordWindow,
}

/* Default initialization values for RecordSettings */
impl Default for RecordSettings {
	fn default() -> Self {
		Self {
			command: None,
			color: 0x003A_A431,
			border: Some(1),
			padding: Padding::default(),
			time: RecordTime::default(),
			flag: RecordFlag::default(),
			window: RecordWindow::Focus(Some(Geometry::default()), false),
		}
	}
}

impl RecordSettings {
	/**
	 * Create a new RecordSettings object.
	 *
	 * @param  command (Option)
	 * @param  color
	 * @param  border (Option)
	 * @param  padding
	 * @param  time
	 * @param  flag
	 * @param  window
	 * @return RecordSettings
	 */
	pub fn new(
		command: Option<&'static str>,
		color: u64,
		border: Option<u32>,
		padding: Padding,
		time: RecordTime,
		flag: RecordFlag,
		window: RecordWindow,
	) -> Self {
		Self {
			command,
			color,
			border,
			padding,
			time,
			flag,
			window,
		}
	}

	/**
	 * Create a new RecordSettings object from arguments.
	 *
	 * @param  matches
	 * @return RecordSettings
	 */
	pub fn from_args(matches: &ArgMatches<'_>) -> Self {
		Self::from_parser(
			ArgParser::from_subcommand(
				matches,
				if matches.is_present("capture") {
					"capture"
				} else {
					"record"
				},
			),
			matches.value_of("color").unwrap_or_default(),
		)
	}

	/**
	 * Create a RecordSettings object from an argument parser.
	 *
	 * @param  parser
	 * @param  color
	 * @return RecordSettings
	 */
	fn from_parser(parser: ArgParser<'_>, color: &str) -> Self {
		match parser.args {
			Some(ref matches) => Self::new(
				match matches.value_of("command") {
					Some(cmd) => Some(Box::leak(cmd.to_string().into_boxed_str())),
					_ => None,
				},
				u64::from_str_radix(color, 16).unwrap_or(Self::default().color),
				match parser.parse("border", 0) {
					border if border > 0 => Some(border),
					_ => None,
				},
				Self::parse_padding(matches),
				RecordTime::from_parser(&parser),
				RecordFlag::new(
					matches.is_present("with-alpha"),
					if matches.is_present("no-keys") {
						None
					} else {
						Some(Box::leak(
							matches
								.value_of("action-keys")
								.unwrap_or_default()
								.to_string()
								.into_boxed_str(),
						))
					},
					Some(Box::leak(
						matches
							.value_of("cancel-keys")
							.unwrap_or_default()
							.to_string()
							.into_boxed_str(),
					)),
					matches.value_of("font").unwrap_or_default(),
					matches.value_of("monitor").and_then(|v| v.parse().ok()),
					if matches.value_of("size").unwrap_or_default().contains('+') {
						matches.is_present("select")
					} else {
						true
					},
					matches.is_present("mouse"),
				),
				RecordWindow::from_args(matches),
			),
			None => RecordSettings::default(),
		}
	}

	/**
	 * Parse the padding value from arguments.
	 *
	 * @param  matches
	 * @return Padding
	 */
	fn parse_padding(matches: &ArgMatches<'_>) -> Padding {
		let mut padding =
			Padding::parse(matches.value_of("padding").unwrap_or_default());
		if matches
			.value_of("size")
			.unwrap_or_default()
			.matches('+')
			.count() == 2
		{
			let mut values = matches
				.value_of("size")
				.unwrap_or_default()
				.split('+')
				.collect::<Vec<&str>>()
				.into_iter()
				.skip(1);
			padding.left = values
				.next()
				.unwrap_or_default()
				.parse()
				.unwrap_or_default();
			padding.top = values
				.next()
				.unwrap_or_default()
				.parse()
				.unwrap_or_default();
		};
		padding
	}

	/**
	 * Get Command from parsed settings.
	 *
	 * @return Command (Option)
	 */
	pub fn get_command<'a>(&self) -> Option<Command<'a>> {
		self.command.map(Command::from)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::{App, Arg};
	use pretty_assertions::assert_eq;
	#[test]
	fn test_record_settings() {
		let args = App::new("test")
			.arg(
				Arg::with_name("action-keys")
					.long("action-keys")
					.takes_value(true),
			)
			.arg(
				Arg::with_name("cancel-keys")
					.long("cancel-keys")
					.takes_value(true),
			)
			.arg(Arg::with_name("border").long("border").takes_value(true))
			.arg(Arg::with_name("padding").long("padding").takes_value(true))
			.arg(Arg::with_name("size").long("size").takes_value(true))
			.arg(
				Arg::with_name("duration")
					.long("duration")
					.takes_value(true),
			)
			.arg(
				Arg::with_name("countdown")
					.long("countdown")
					.takes_value(true),
			)
			.arg(Arg::with_name("timeout").long("timeout").takes_value(true))
			.arg(
				Arg::with_name("interval")
					.long("interval")
					.takes_value(true),
			)
			.arg(Arg::with_name("root").long("root"))
			.arg(Arg::with_name("focus").long("focus"))
			.arg(Arg::with_name("with-alpha").long("with-alpha"))
			.arg(Arg::with_name("no-keys").long("no-keys"))
			.get_matches_from(vec![
				"test",
				"--action-keys",
				"LControl-Q,S",
				"--cancel-keys",
				"X",
				"--border",
				"10",
				"--padding",
				"0:0:0:0",
				"--size",
				"10x10+10+10",
				"--duration",
				"1",
				"--countdown",
				"2",
				"--timeout",
				"300",
				"--interval",
				"12",
				"--root",
				"--with-alpha",
			]);
		let record_settings =
			RecordSettings::from_parser(ArgParser::from_args(&args), "000000");
		assert_eq!(0x0000_0000, record_settings.color);
		assert_eq!(10, record_settings.border.unwrap());
		assert_eq!(Padding::new(10, 0, 0, 10), record_settings.padding);
		assert_eq!(2, record_settings.time.countdown);
		assert_eq!(300, record_settings.time.timeout);
		assert_eq!(12, record_settings.time.interval);
		assert_eq!(
			RecordWindow::Root(Some(Geometry::new(0, 0, 10, 10))),
			record_settings.window
		);
		assert!(record_settings.flag.alpha);
		assert_eq!("LControl-Q,S", record_settings.flag.action_keys.unwrap());
		assert_eq!("X", record_settings.flag.cancel_keys.unwrap());
	}
}
