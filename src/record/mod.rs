pub mod fps;
use crate::image::gif::Frame;
use crate::image::Image;
use crate::record::fps::{FpsClock, TimeUnit};
use std::sync::mpsc;
use std::thread;

/* Sender and main thread of the Recorder */
pub struct RecordResult {
	pub sender: mpsc::Sender<()>,
	pub thread: thread::JoinHandle<Vec<Frame>>,
}

impl RecordResult {
	/**
	 * Create a new RecordResult object.
	 *
	 * @param  sender
	 * @param  thread
	 * @return RecordResult
	 */
	pub fn new(
		sender: mpsc::Sender<()>,
		thread: thread::JoinHandle<Vec<Frame>>,
	) -> Self {
		Self { sender, thread }
	}
}

/* Recorder with FPS clock and channel */
pub struct Recorder {
	clock: FpsClock,
	channel: (mpsc::Sender<()>, mpsc::Receiver<()>),
}

impl Recorder {
	/**
	 * Create a new Recorder object.
	 *
	 * @param  fps
	 * @return Recorder
	 */
	pub fn new(fps: u32) -> Self {
		Self {
			clock: FpsClock::new(fps),
			channel: mpsc::channel(),
		}
	}

	/**
	 * Start recording the frames.
	 *
	 * @param  get_image (Fn)
	 * @return RecordResult
	 */
	pub fn record(
		mut self,
		get_image: impl Fn() -> Option<Image> + Sync + Send + 'static,
	) -> RecordResult {
		let mut frames = Vec::new();
		RecordResult::new(
			self.channel.0.clone(),
			thread::spawn(move || {
				while self.channel.1.try_recv().is_err() {
					frames.push(Frame::new(
						get_image().unwrap(),
						(self.clock.get_fps(TimeUnit::Millisecond) / 10.) as u16,
					));
					self.clock.tick();
				}
				frames
			}),
		)
	}
}
