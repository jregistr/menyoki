use crate::encode::gif::{Frame, Gif};
use crate::encode::Image;
use crate::record::Recorder;
use crate::settings::AppSettings;
use std::fs::File;
use std::io::Error;

/* Application and main functionalities */
pub struct App {
	settings: AppSettings,
}

impl App {
	/**
	 * Create a new App object.
	 *
	 * @param  settings
	 * @return App
	 */
	pub fn new(settings: AppSettings) -> Self {
		Self { settings }
	}

	/**
	 * Start recording the frames.
	 *
	 * @param  get_image (Fn)
	 * @return Vector of Frame
	 */
	pub fn record(
		&self,
		get_image: impl Fn() -> Option<Image> + Sync + Send + 'static,
	) -> Vec<Frame> {
		let mut recorder =
			Recorder::new(self.settings.get_record_settings(), get_image);
		if self.settings.args.is_present("command") {
			let record = recorder.record_async();
			self.settings
				.get_command()
				.execute()
				.expect("Failed to run the command");
			record.finish().expect("Failed to finish the recording");
			record.thread.join().expect("Failed to retrieve the frames")
		} else {
			recorder.record_sync()
		}
	}

	/**
	 * Save frames to a GIF file.
	 *
	 * @param  frames
	 * @return Result
	 */
	pub fn save_gif(&self, frames: Vec<Frame>) -> Result<(), Error> {
		let mut gif = Gif::new(
			File::create(self.settings.get_output_file())
				.expect("Failed to create file"),
			frames
				.first()
				.expect("No frames found to save")
				.image
				.geometry,
			self.settings.get_gif_settings(),
		)?;
		gif.save(frames)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::encode::Geometry;
	use crate::util;
	use crate::util::cmd::Command;
	#[test]
	fn test_app_mod() -> Result<(), Error> {
		let settings = AppSettings::new(util::parse_args());
		let app = App::new(settings);
		let geometry = Geometry::new(0, 0, 1, 1);
		let mut frames =
			app.record(move || Some(Image::new(vec![0, 0, 0], geometry)));
		frames.push(Frame::new(Image::new(vec![255, 255, 255], geometry), 0));
		app.save_gif(frames)?;
		Command::new(String::from("rm"), vec![String::from("t.gif")]).execute()?;
		Ok(())
	}
}
