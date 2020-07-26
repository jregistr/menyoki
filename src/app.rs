use crate::gif::decoder::Decoder;
use crate::gif::encoder::{Encoder, Frames};
#[cfg(feature = "ski")]
use crate::gif::ski::Gif;
#[cfg(not(feature = "ski"))]
use crate::gif::Gif;
use crate::image::Image;
use crate::record::{Record, Recorder};
use crate::settings::AppSettings;
use crate::util::file::FileFormat;
use bytesize::ByteSize;
use image::bmp::BMPEncoder;
use image::farbfeld::FarbfeldEncoder;
use image::io::Reader;
use image::jpeg::JPEGEncoder;
use image::png::PNGEncoder;
use image::tiff::TiffEncoder;
use image::ColorType;
use image::ImageEncoder;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{Error, Read, Seek, Write};
use std::thread;

/* Window system functions */
pub trait WindowAccess<'a, Window: Record + Send + Sync + Copy + Debug + 'static> {
	fn init(settings: &'a AppSettings<'a>) -> Option<Self>
	where
		Self: Sized;
	fn get_window(&mut self) -> Option<Window>;
}

/* Application output (Image or Frames) */
type AppOutput = (Option<Image>, Option<Frames>);

/* Application and main functionalities */
#[derive(Clone, Copy, Debug)]
pub struct App<'a, Window> {
	window: Option<Window>,
	settings: &'a AppSettings<'a>,
}

impl<'a, Window> App<'a, Window>
where
	Window: Record + Send + Sync + Copy + Debug + 'static,
{
	/**
	 * Create a new App object.
	 *
	 * @param  window (Option)
	 * @param  settings
	 * @return App
	 */
	pub fn new(window: Option<Window>, settings: &'a AppSettings<'a>) -> Self {
		Self { window, settings }
	}

	/**
	 * Start the application.
	 *
	 * @param  output
	 * @return Result
	 */
	pub fn start<Output: Write + Seek>(
		&self,
		mut output: Output,
	) -> Result<(), Error> {
		trace!("{:?}", self.window);
		debug!("{:?}", self.settings.save.file);
		debug!("Command: {:?}", self.settings.get_command());
		let (image, frames) = self.get_output();
		match self.settings.save.file.format {
			FileFormat::Gif => {
				debug!("{:?}", self.settings.gif);
				self.save_gif(frames, output)?;
			}
			FileFormat::Png => {
				debug!("{:?}", self.settings.png);
				self.save_image(
					image,
					PNGEncoder::new_with_quality(
						output,
						self.settings.png.compression,
						self.settings.png.filter,
					),
					ColorType::Rgba8,
				)
			}
			FileFormat::Jpg => {
				debug!("{:?}", self.settings.jpg);
				self.save_image(
					image,
					JPEGEncoder::new_with_quality(
						&mut output,
						self.settings.jpg.quality,
					),
					ColorType::Rgb8,
				)
			}
			FileFormat::Bmp => self.save_image(
				image,
				BMPEncoder::new(&mut output),
				ColorType::Rgba8,
			),
			FileFormat::Tiff => {
				self.save_image(image, TiffEncoder::new(output), ColorType::Rgba8)
			}
			FileFormat::Ff => self.save_image(
				image,
				FarbfeldEncoder::new(output),
				ColorType::Rgba16,
			),
		}
		info!(
			"{} saved to: {:?} ({})",
			self.settings.save.file.format.to_string().to_uppercase(),
			self.settings.save.file.path,
			ByteSize(fs::metadata(&self.settings.save.file.path)?.len())
		);
		if let Some(window) = self.window {
			window.release();
		}
		Ok(())
	}

	/**
	 * Get the application output.
	 *
	 * @return AppOutput
	 */
	fn get_output(&self) -> AppOutput {
		if self.settings.save.file.format == FileFormat::Gif {
			(None, Some(self.get_frames()))
		} else {
			(self.get_image(), None)
		}
	}

	/**
	 * Get the image to save.
	 *
	 * @return Image (Option)
	 */
	fn get_image(&self) -> Option<Image> {
		if self.settings.args.is_present("edit") {
			info!("Opening {:?}...", self.settings.edit.path);
			Some(self.edit_image())
		} else {
			self.capture()
		}
	}

	/**
	 * Get the frames to save.
	 *
	 * @return Frames
	 */
	fn get_frames(&self) -> Frames {
		if self.settings.args.is_present("edit") {
			info!("Reading the frames from {:?}...", self.settings.edit.path);
			self.edit_gif(
				File::open(self.settings.edit.path).expect("File not found"),
			)
		} else {
			(self.record(), self.settings.record.fps)
		}
	}

	/**
	 * Capture the image of window.
	 *
	 * @return Image (Option)
	 */
	fn capture(self) -> Option<Image> {
		let window = self.window.expect("Failed to get the window");
		if self.settings.args.is_present("command") {
			let image_thread = thread::spawn(move || {
				window.show_countdown();
				info!("Capturing an image...");
				window.get_image()
			});
			self.settings
				.get_command()
				.expect("No command specified to run")
				.execute()
				.expect("Failed to run the command");
			image_thread
				.join()
				.expect("Failed to join the image thread")
		} else {
			window.show_countdown();
			info!("Capturing an image...");
			window.get_image()
		}
	}

	/**
	 * Start recording the frames.
	 *
	 * @return Vector of Image
	 */
	fn record(self) -> Vec<Image> {
		let mut recorder = Recorder::new(
			self.window.expect("Failed to get the window"),
			self.settings.record,
		);
		if self.settings.args.is_present("command") {
			let record = recorder.record_async();
			self.settings
				.get_command()
				.expect("No command specified to run")
				.execute()
				.expect("Failed to run the command");
			match record.get() {
				Some(frames) => frames.expect("Failed to retrieve the frames"),
				None => Vec::new(),
			}
		} else {
			recorder.record_sync(&self.settings.input_state)
		}
	}

	/**
	 * Edit and return the image.
	 *
	 * @return Image
	 */
	fn edit_image(self) -> Image {
		let image = Reader::open(self.settings.edit.path)
			.expect("File not found")
			.with_guessed_format()
			.expect("File format not supported")
			.decode()
			.expect("Failed to decode the image")
			.to_rgba();
		let mut imageops = self.settings.edit.get_imageops();
		imageops.init(image.dimensions());
		imageops.process(image).get_image()
	}

	/**
	 * Return the updated frames after decoding the GIF.
	 *
	 * @param  input
	 * @return Frames
	 */
	fn edit_gif<Input: Read>(self, input: Input) -> Frames {
		Decoder::new(input, self.settings.edit.get_imageops(), self.settings.gif)
			.expect("Failed to decode the GIF")
			.update_frames()
			.expect("Failed to edit the GIF")
	}

	/**
	 * Save the image to a file.
	 *
	 * @param image (Option)
	 * @param encoder
	 * @param color_type
	 */
	fn save_image<Encoder: ImageEncoder>(
		&self,
		image: Option<Image>,
		encoder: Encoder,
		color_type: ColorType,
	) {
		let image = image.expect("Failed to get the window image");
		info!(
			"Saving the image as {}...",
			self.settings.save.file.format.to_string().to_uppercase()
		);
		debug!("{:?}", image);
		debug!("Color type: {:?}", color_type);
		encoder
			.write_image(
				&image.get_data(color_type),
				image.geometry.width,
				image.geometry.height,
				color_type,
			)
			.expect("Failed to encode the image");
	}

	/**
	 * Save frames to a GIF file.
	 *
	 * @param  frames (Option)
	 * @param  output
	 * @return Result
	 */
	fn save_gif<Output: Write>(
		self,
		frames: Option<Frames>,
		output: Output,
	) -> Result<(), Error> {
		let (images, fps) = frames.expect("Failed to get the frames");
		debug!("FPS: {}", fps);
		Gif::new(
			fps,
			images.first().expect("No frames found to save").geometry,
			output,
			self.settings.gif,
		)?
		.save(images, &self.settings.input_state)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::args::Args;
	use crate::image::Image;
	use crate::test::TestWindow;
	use crate::util::file::FileFormat;
	use image::Bgra;
	use std::io::Cursor;
	#[test]
	fn test_app_mod() -> Result<(), Error> {
		let args = Args::parse();
		let mut settings = AppSettings::new(&args);
		let window = TestWindow::default();
		for format in vec![
			FileFormat::Png,
			FileFormat::Jpg,
			FileFormat::Bmp,
			FileFormat::Tiff,
			FileFormat::Ff,
		] {
			settings.save.file.format = format;
			let app = App::new(Some(window), &settings);
			let mut output = Vec::new();
			app.start(Cursor::new(&mut output))?;
			assert!(output.len() > 0);
		}
		settings.save.file.format = FileFormat::Gif;
		let app = App::new(Some(window), &settings);
		let mut images = app.record();
		images.push(Image::new(
			vec![Bgra::from([0, 0, 0, 0])],
			false,
			window.geometry,
		));
		let mut output = Vec::new();
		app.save_gif((images, 10), &mut output)?;
		assert!(output.len() > 0);
		Ok(())
	}
}
