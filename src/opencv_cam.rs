use std::thread;

use opencv::aruco::{calibrate_camera_charuco, calibrate_camera_charuco_def};
use opencv::core::Ptr;
use opencv::prelude::MatTraitConst;
use opencv::{
    aruco::interpolate_corners_charuco_def,
    core::{Point2f, Point3f, Scalar, Size, Vector},
    imgcodecs,
    objdetect::{
        CharucoBoard, CharucoDetector, Dictionary, draw_detected_markers,
        draw_detected_markers_def, get_predefined_dictionary,
    },
    prelude::{
        BoardTraitConst, CharucoBoardTraitConst, CharucoDetectorTraitConst,
        DictionaryTraitConst,
    },
};

use opencv::core::Mat;
use serde::{Deserialize, Serialize};

use crate::{CameraStream, CameraStreamConfig, VideoSourceConfig};

pub type CharucoMarker = (Vector<Vector<Point2f>>, Vector<i32>);

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenCvCameraConfig {
    pub camera_stream_config: CameraStreamConfig,
    // pub charuco_board: CharucoBoard,
}

#[derive(Debug)]
pub struct OpenCvCamera {
    // pub stream: CameraStream,
    charuco_board: Option<CharucoBoard>,
    r: tokio::sync::watch::Receiver<Mat>,
    r_charuco_markers: tokio::sync::watch::Receiver<CharucoMarker>,
    pub camera_stream_config: CameraStreamConfig,
}

impl TryFrom<OpenCvCameraConfig> for OpenCvCamera {
    type Error = anyhow::Error;

    fn try_from(config: OpenCvCameraConfig) -> Result<Self, Self::Error> {
        // let stream = config.camera_stream_config.try_into()?;
        let (s, r) = tokio::sync::watch::channel(Mat::default());
        let (s_charuco_markers, r_charuco_markers) =
            tokio::sync::watch::channel((Vector::new(), Vector::new()));

        Ok(Self {
            // stream,
            charuco_board: None,
            r,
            r_charuco_markers,
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

        let charuco_board_clone = charuco_board.clone();

        let (s, r) = tokio::sync::watch::channel(Mat::default());
        let (s_charuco_markers, r_charuco_markers) =
            tokio::sync::watch::channel((Vector::new(), Vector::new()));

        thread::spawn(move || {
            let charuco_detector =
                CharucoDetector::new_def(&charuco_board_clone)
                    .expect("Failed to create charuco detector");

            loop {
                Self::update(
                    &stream,
                    &charuco_detector,
                    &charuco_board_clone,
                    &s,
                    &s_charuco_markers,
                );

                // thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            // stream,
            charuco_board: Some(charuco_board),
            r,
            r_charuco_markers,
            camera_stream_config,
        }
    }

    pub fn get_latest_frame(&self) -> Mat {
        self.r.borrow().clone()
    }

    pub fn get_latest_charuco_markers(&self) -> CharucoMarker {
        self.r_charuco_markers.borrow().clone()
    }

    fn update(
        stream: &CameraStream,
        charuco_detector: &CharucoDetector,
        charuco_board: &CharucoBoard,
        s: &tokio::sync::watch::Sender<Mat>,
        s_charuco_markers: &tokio::sync::watch::Sender<CharucoMarker>,
    ) {
        let mut frame = stream.get_latest_image();

        let (marker_corners, marker_ids, charuco_corners, charuco_ids) =
            Self::detect_charuco(&frame, charuco_detector, charuco_board)
                .expect("Failed to detect charuco");

        // draw_detected_markers(
        //     &mut frame,
        //     &marker_corners,
        //     &marker_ids,
        //     Scalar::new(0.0, 255.0, 0.0, 0.0),
        // )
        // .expect("Failed to draw detected markers");

        s.send(frame).expect("Failed to send frame");
        s_charuco_markers
            .send((marker_corners, marker_ids))
            .expect("Failed to send charuco corners");
    }

    fn detect_charuco(
        frame: &Mat,
        charuco_detector: &CharucoDetector,
        charuco_board: &CharucoBoard,
    ) -> Result<(Vector<Vector<Point2f>>, Vector<i32>, Mat, Mat), opencv::Error>
    {
        let mut charuco_corners = Mat::default();
        let mut charuco_ids = Mat::default();
        let mut marker_corners = Vector::<Vector<Point2f>>::new();
        let mut marker_ids = Vector::<i32>::new();

        charuco_detector.detect_board(
            frame,
            &mut charuco_corners,
            &mut charuco_ids,
            &mut marker_corners,
            &mut marker_ids,
        )?;

        // 検知できたマーカーとcharucoボードの幾何から残りのマーカーを補完する
        if marker_ids.len() > 0 {
            interpolate_corners_charuco_def(
                &mut marker_corners,
                &mut marker_ids,
                frame,
                &Ptr::new(charuco_board.clone()),
                &mut charuco_corners,
                &mut charuco_ids,
            )?;
        }

        Ok((marker_corners, marker_ids, charuco_corners, charuco_ids))
    }
}

fn save_mat_as_image(mat: Mat) {
    imgcodecs::imwrite_def("charuco.png", &mat).unwrap();
    println!("Saved charuco.png");
}
