use eframe::egui;
use opencv::{
    core::{Mat, MatTraitConst, MatTraitConstManual},
    videoio::{VideoCapture, VideoCaptureTrait},
};
use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::VideoSource;

pub struct ThreadingCam {
    join_handle: thread::JoinHandle<()>,
    shared_img: Arc<Mutex<Option<egui::ColorImage>>>,
}

impl ThreadingCam {
    pub fn new(mut vsrc: impl VideoSource + Send + 'static) -> Self {
        let shared_img = Arc::new(Mutex::new(None));
        let shared_img_clone = shared_img.clone();

        let join_handle = thread::spawn(move || {
            loop {
                if let Ok(frame) = vsrc.read() {
                    // let mut rgb = Mat::default();
                    // opencv::imgproc::cvt_color(
                    //     &frame,
                    //     &mut rgb,
                    //     opencv::imgproc::COLOR_BGR2RGB,
                    //     0,
                    //     opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
                    // )
                    // .unwrap();
                    // let mut scaled = Mat::default();
                    // opencv::imgproc::resize(
                    //     &rgb,
                    //     &mut scaled,
                    //     opencv::core::Size {
                    //         width: 1920,
                    //         height: 1080,
                    //     },
                    //     0.0,
                    //     0.0,
                    //     opencv::imgproc::INTER_CUBIC,
                    // )
                    // .unwrap();
                    // frame = scaled;

                    // let frame.
                    let frame_size =
                        [frame.cols() as usize, frame.rows() as usize];

                    let img = egui::ColorImage::from_rgb(
                        frame_size,
                        frame.data_bytes().unwrap(),
                    );

                    let mut shared_img_lock = shared_img_clone.lock().unwrap();
                    *shared_img_lock = Some(img);
                }
            }
        });

        Self {
            join_handle,
            shared_img,
        }
    }

    pub fn get_latest_image(&self) -> Option<egui::ColorImage> {
        let img_lock = self.shared_img.lock().unwrap();
        img_lock.clone()
    }
}

impl Drop for ThreadingCam {
    fn drop(&mut self) {}
}
