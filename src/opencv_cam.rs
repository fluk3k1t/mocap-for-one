use std::thread;

use opencv::{
    core::Size,
    imgcodecs,
    objdetect::{CharucoBoard, Dictionary, get_predefined_dictionary},
    prelude::{BoardTraitConst, CharucoBoardTraitConst, DictionaryTraitConst},
};

use opencv::core::Mat;
use serde::{Deserialize, Serialize};

use crate::{CameraStream, CameraStreamConfig, VideoSourceConfig};

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenCvCameraConfig {
    pub camera_stream_config: CameraStreamConfig,
    // pub charuco_board: CharucoBoard,
}

#[derive(Debug)]
pub struct OpenCvCamera {
    // pub stream: CameraStream,
    charuco_board: Option<CharucoBoard>,
    r: tokio::sync::watch::Receiver<Option<Mat>>,
    pub camera_stream_config: CameraStreamConfig,
}

impl TryFrom<OpenCvCameraConfig> for OpenCvCamera {
    type Error = anyhow::Error;

    fn try_from(config: OpenCvCameraConfig) -> Result<Self, Self::Error> {
        // let stream = config.camera_stream_config.try_into()?;
        let (s, r) = tokio::sync::watch::channel(None);

        Ok(Self {
            // stream,
            charuco_board: None,
            r,
            camera_stream_config: config.camera_stream_config,
        })
    }
}

impl From<&OpenCvCamera> for OpenCvCameraConfig {
    fn from(camera: &OpenCvCamera) -> Self {
        Self {
            camera_stream_config: camera.camera_stream_config.clone(),
        }
    }
}

impl OpenCvCamera {
    pub fn new(stream: CameraStream) -> Self {
        // thread::spawn(|| loop {
        //     if let Ok(frame) = stream.read() {

        //     }
        // });
        let camera_stream_config = (&stream).into();

        let charuco_board = CharucoBoard::new_def(
            Size::new(3, 3),
            0.3,
            0.15,
            &get_predefined_dictionary(
                opencv::objdetect::PredefinedDictionaryType::DICT_6X6_250,
            )
            .unwrap(),
        )
        .unwrap();

        // println!("here");

        // https://docs.opencv.org/4.x/df/d4a/tutorial_charuco_detection.html
        let mut img = Mat::default();
        charuco_board
            .generate_image_def(
                Size::new(
                    (charuco_board.get_chessboard_size().unwrap().width as f32
                        * charuco_board.get_square_length().unwrap() as f32
                        * 1000.0
                        * 13.8)
                        .floor() as i32,
                    (charuco_board.get_chessboard_size().unwrap().height as f32
                        * charuco_board.get_square_length().unwrap() as f32
                        * 1000.0
                        * 13.8)
                        .floor() as i32,
                ),
                &mut img,
            )
            .unwrap();
        // img.
        // save_mat_as_image(img);
        // println!("ok");

        let (s, r) = tokio::sync::watch::channel(None);
        thread::spawn(move || {
            loop {
                if let Some(frame) = stream.get_latest_image() {
                    // s.send(frame).unwrap();
                }
            }
        });

        Self {
            // stream,
            charuco_board: Some(charuco_board),
            r,
            camera_stream_config,
        }
    }

    pub fn get_latest_frame(&self) -> Option<Mat> {
        self.r.borrow().clone()
    }
}

fn save_mat_as_image(mat: Mat) {
    imgcodecs::imwrite_def("charuco.png", &mat).unwrap();
    println!("Saved charuco.png");
}
