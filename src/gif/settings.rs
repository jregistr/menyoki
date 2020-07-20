use crate::args::parser::ArgParser;

/* GIF and frame settings */
#[derive(Clone, Copy, Debug)]
pub struct GifSettings<'a> {
	pub file: &'a str,
	pub repeat: i32,
	pub quality: u8,
	pub speed: f32,
	pub fast: bool,
}

/* Default initialization values for GifSettings */
impl Default for GifSettings<'_> {
	fn default() -> Self {
		Self {
			file: "",
			repeat: -1,
			quality: 75,
			speed: 1.,
			fast: false,
		}
	}
}

impl<'a> GifSettings<'a> {
	/**
	 * Create a new GifSettings object.
	 *
	 * @param  file
	 * @param  repeat
	 * @param  quality
	 * @param  speed
	 * @param  fast
	 * @return GifSettings
	 */
	pub fn new(
		file: &'a str,
		repeat: i32,
		quality: u8,
		speed: f32,
		fast: bool,
	) -> Self {
		if quality <= 20 {
			warn!("GIF will be encoded in low quality.");
		}
		Self {
			file,
			repeat,
			quality,
			speed,
			fast,
		}
	}

	/**
	 * Create a GifSettings object from parsed arguments.
	 *
	 * @param  parser
	 * @return GifSettings
	 */
	pub fn from_args(parser: ArgParser<'a>) -> Self {
		match parser.args {
			Some(matches) => Self::new(
				matches.value_of("input").unwrap_or_default(),
				parser.parse("repeat", Self::default().repeat) - 1,
				parser.parse("quality", Self::default().quality),
				parser.parse("speed", Self::default().speed),
				matches.is_present("fast"),
			),
			None => Self::default(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::{App, Arg};
	#[test]
	fn test_gif_settings() {
		let args = App::new("test")
			.arg(Arg::with_name("repeat").long("repeat").takes_value(true))
			.arg(Arg::with_name("quality").long("quality").takes_value(true))
			.get_matches_from(vec!["test", "--repeat", "5", "--quality", "10"]);
		let gif_settings = GifSettings::from_args(ArgParser::new(Some(&args)));
		assert_eq!(4, gif_settings.repeat);
		assert_eq!(10, gif_settings.quality);
		let gif_settings = GifSettings::from_args(ArgParser::new(None));
		assert_eq!(-1, gif_settings.repeat);
		assert_eq!(75, gif_settings.quality);
	}
}
