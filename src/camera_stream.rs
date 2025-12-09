use anyhow::Result;
use eframe::egui::{self, ColorImage};
use opencv::core::{MatTraitConst, MatTraitConstManual};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::watch;

use crate::{VideoSource, VideoSourceConfig};

#[derive(Debug)]
pub struct CameraStream {
    join_handle: thread::JoinHandle<()>,
    pub name: String,
    // shared_img: Arc<Mutex<Option<egui::ColorImage>>>,
    r: watch::Receiver<Option<ColorImage>>,
    pub video_source_config: VideoSourceConfig,
}

impl CameraStream {
    pub fn new(name: String, config: VideoSourceConfig) -> Result<Self> {
        // let shared_img = Arc::new(Mutex::new(None));
        // let shared_img_clone = Arc::clone(&shared_img);
        let video_source_config = config.clone();
        let mut vsrc = VideoSource::try_from(config)?;
        let (s, r) = watch::channel(None);

        let join_handle = thread::spawn(move || {
            loop {
                if let Ok(frame) = vsrc.read() {
                    let frame_size =
                        [frame.cols() as usize, frame.rows() as usize];

                    if let Ok(raw) = frame.data_bytes() {
                        let img = egui::ColorImage::from_rgb(frame_size, raw);
                        // s.send(Some(img)).unwrap();
                        if let Err(err) = s.send(Some(img)) {
                            eprintln!("Failed to send image: {}", err);
                        }
                    }
                }
            }
        });

        Ok(Self {
            name,
            join_handle,
            r: r.clone(),
            video_source_config,
        })
    }

    pub fn latest_image(&self) -> Option<egui::ColorImage> {
        // self.shared_img.lock().ok().and_then(|img_lock| (*img_lock).clone())
        self.r.borrow().clone()
    }
}
