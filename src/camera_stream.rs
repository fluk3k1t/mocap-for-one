use anyhow::Result;
use eframe::egui::{self, ColorImage};
use opencv::core::{MatTraitConst, MatTraitConstManual};
use serde::{Deserialize, Serialize};
use std::thread;
use tokio::sync::watch;

use crate::{VideoSource, VideoSourceConfig};

#[derive(Debug)]
pub struct CameraStream {
    pub name: String,
    r: watch::Receiver<Option<ColorImage>>,
    pub video_source_config: VideoSourceConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CameraStreamConfig {
    pub name: String,
    pub video_source_config: VideoSourceConfig,
}

impl CameraStream {
    pub fn new(name: String, config: VideoSourceConfig) -> Result<Self> {
        let video_source_config = config.clone();
        let mut vsrc = VideoSource::try_from(config)?;
        let (s, r) = watch::channel(None);

        // rustのthreadはjoin handleをとらなければ自動的にデタッチされ
        // threadが終了した時点でリソースが解放される
        // CameraStreamがdropされたらchannelが閉じられてfinishするのでメモリリークしないはず
        let _ = thread::spawn(move || {
            loop {
                if let Ok(frame) = vsrc.read() {
                    let frame_size =
                        [frame.cols() as usize, frame.rows() as usize];

                    if let Ok(raw) = frame.data_bytes() {
                        let img = egui::ColorImage::from_rgb(frame_size, raw);
                        // abort thread when CameraStream is dropped
                        if let Err(_) = s.send(Some(img)) {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            name,
            r: r.clone(),
            video_source_config,
        })
    }

    pub fn get_latest_image(&self) -> Option<egui::ColorImage> {
        self.r.borrow().clone()
    }
}

impl TryFrom<CameraStreamConfig> for CameraStream {
    type Error = anyhow::Error;

    fn try_from(value: CameraStreamConfig) -> Result<Self, Self::Error> {
        Self::new(value.name, value.video_source_config)
    }
}

impl From<&CameraStream> for CameraStreamConfig {
    fn from(value: &CameraStream) -> Self {
        Self {
            name: value.name.clone(),
            video_source_config: value.video_source_config.clone(),
        }
    }
}
