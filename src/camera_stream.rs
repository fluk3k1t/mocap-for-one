use anyhow::Result;
use eframe::egui;
use opencv::core::{MatTraitConst, MatTraitConstManual};
use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{VideoSource, VideoSourceConfig};

pub struct CameraStream {
    join_handle: thread::JoinHandle<()>,
    shared_img: Arc<Mutex<Option<egui::ColorImage>>>,
    pub video_source_config: VideoSourceConfig,
}

impl CameraStream {
    pub fn new(config: VideoSourceConfig) -> Result<Self> {
        let shared_img = Arc::new(Mutex::new(None));
        let shared_img_clone = Arc::clone(&shared_img);
        let video_source_config = config.clone();
        let mut vsrc = VideoSource::try_from(config)?;

        let join_handle = thread::spawn(move || {
            loop {
                if let Ok(frame) = vsrc.read() {
                    let frame_size =
                        [frame.cols() as usize, frame.rows() as usize];

                    if let Ok(raw) = frame.data_bytes() {
                        let img = egui::ColorImage::from_rgb(frame_size, raw);
                        if let Ok(mut shared_img_lock) = shared_img_clone.lock()
                        {
                            *shared_img_lock = Some(img);
                        }
                    }
                }
            }
        });

        Ok(Self {
            join_handle,
            shared_img,
            video_source_config,
        })
    }

    pub fn latest_image(&self) -> Option<egui::ColorImage> {
        self.shared_img.lock().ok().and_then(|img_lock| (*img_lock).clone())
    }
}
