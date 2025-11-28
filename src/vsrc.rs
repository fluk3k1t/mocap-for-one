use memmap2::Mmap;
use opencv::core::{Mat, Vector};
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use std::fs::File;
use std::path::Path;

pub trait VideoSource {
    fn read(&mut self) -> Result<Mat, ()>;
}

pub struct MMAPCam {
    mmap: Mmap,
}

impl MMAPCam {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        // Safety: We assume the file is not concurrently modified in a way that causes UB,
        // though for IPC with mmap, race conditions on content are expected but usually safe for read-only mapping of data.
        // However, if the file is truncated while mapped, it can cause SIGBUS.
        // The Python code uses it for reading images, presumably written by another process.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }
}

impl VideoSource for MMAPCam {
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

impl VideoSource for VideoCapture {
    fn read(&mut self) -> Result<Mat, ()> {
        let mut frame = Mat::default();
        let success =
            VideoCaptureTrait::read(self, &mut frame).map_err(|_| ())?;

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
