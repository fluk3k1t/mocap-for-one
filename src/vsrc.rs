use anyhow::anyhow;
use memmap2::Mmap;
use opencv::core::Mat;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

pub struct MMAPCam {
    mmap: Mmap,
    path: String,
}

impl MMAPCam {
    pub fn new<P: AsRef<Path> + Clone>(path: P) -> std::io::Result<Self> {
        let file = File::open(path.clone())?;
        // Safety: We assume the file is not concurrently modified in a way that causes UB,
        // though for IPC with mmap, race conditions on content are expected but usually safe for read-only mapping of data.
        // However, if the file is truncated while mapped, it can cause SIGBUS.
        // The Python code uses it for reading images, presumably written by another process.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self {
            mmap,
            path: path.as_ref().to_string_lossy().into_owned(),
        })
    }

    fn read(&mut self) -> Result<Mat, ()> {
        // Create a Mat from the mmap slice (as a byte buffer)
        // We use from_slice to create a 1D Mat of bytes
        let bytes_mat = Mat::from_slice(&self.mmap).map_err(|_| ())?;

        // Decode the image
        // IMREAD_COLOR loads as BGR
        let frame = imgcodecs::imdecode(&bytes_mat, imgcodecs::IMREAD_COLOR)
            .map_err(|_| ())?;

        if frame.empty() {
            return Err(());
        }

        // Convert BGR to RGB to match Python's PIL .convert('RGB')
        let mut rgb_frame = Mat::default();
        imgproc::cvt_color(
            &frame,
            &mut rgb_frame,
            imgproc::COLOR_BGR2RGB,
            0,
            opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )
        .map_err(|_| ())?;

        Ok(rgb_frame)
    }
}

/// Enum holding either MMAPCam or VideoCapture
pub enum VideoSource {
    MMAP(MMAPCam),
    Capture(VideoCapture),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoSourceConfig {
    MMAP { path: String },
    Capture { index: i32 },
}

impl TryFrom<VideoSourceConfig> for VideoSource {
    type Error = anyhow::Error;

    fn try_from(config: VideoSourceConfig) -> Result<Self, Self::Error> {
        match config {
            VideoSourceConfig::MMAP { path } => {
                let mmap_cam = MMAPCam::new(path)?;
                Ok(VideoSource::MMAP(mmap_cam))
            }
            VideoSourceConfig::Capture { index } => {
                let mut vcap =
                    VideoCapture::new(index, opencv::videoio::CAP_ANY)?;
                if !vcap.is_opened()? {
                    return Err(anyhow!(
                        "Failed to open camera at index {index}"
                    ));
                }
                Ok(VideoSource::Capture(vcap))
            }
        }
    }
}

impl VideoSource {
    pub fn read(&mut self) -> Result<Mat, ()> {
        match self {
            VideoSource::MMAP(mmap) => mmap.read(),
            VideoSource::Capture(vcap) => {
                let mut frame = Mat::default();
                let success = VideoCaptureTrait::read(vcap, &mut frame)
                    .map_err(|_| ())?;

                if !success || frame.empty() {
                    return Err(());
                }

                let mut rgb_frame = Mat::default();
                imgproc::cvt_color(
                    &frame,
                    &mut rgb_frame,
                    imgproc::COLOR_BGR2RGB,
                    0,
                    opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
                )
                .map_err(|_| ())?;

                Ok(rgb_frame)
            }
        }
    }
}

impl Drop for VideoSource {
    fn drop(&mut self) {
        match self {
            VideoSource::MMAP(_) => {}
            VideoSource::Capture(vcap) => {
                vcap.release().expect("Failed to release video capture");
            }
        }
    }
}
