use crate::{
    CameraParameter, CameraParameterNum, CameraStream, CharucoMarker,
    OpenCvCamera, OpenCvCameraConfig, VideoSourceConfig, camera_stream,
};
use anyhow::Result;
use opencv::core::Size;
use opencv::{
    aruco::{calibrate_camera_charuco, calibrate_camera_charuco_def},
    core::{Mat, MatTraitConst, Point2f, Ptr, Vector},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkLoadConfig {
    pub opencv_cams: Vec<OpenCvCameraConfig>,
}

pub struct WorkLoad {
    pub opencv_cams: Vec<OpenCvCameraModel>,
}

impl TryFrom<WorkLoadConfig> for WorkLoad {
    type Error = anyhow::Error;

    fn try_from(config: WorkLoadConfig) -> Result<Self, Self::Error> {
        let opencv_cams = config
            .opencv_cams
            .into_iter()
            .map(TryInto::try_into)
            .map(|c| OpenCvCameraModel::new(c.unwrap()))
            .collect();
        Ok(Self { opencv_cams })
    }
}

impl From<&WorkLoad> for WorkLoadConfig {
    fn from(workload: &WorkLoad) -> Self {
        Self {
            opencv_cams: workload
                .opencv_cams
                .iter()
                .map(|c| (&c.opencv_camera).into())
                .collect(),
        }
    }
}

impl WorkLoad {
    pub fn new() -> Self {
        Self {
            opencv_cams: vec![],
        }
    }

    pub fn add_camera_stream(&mut self, stream: CameraStream) {
        self.opencv_cams
            .push(OpenCvCameraModel::new(OpenCvCamera::new(stream)));
    }
}

#[derive(Clone)]
struct FrameAnnotated {
    frame: Mat,
    charuco_corners: Mat,
    charuco_ids: Mat,
    marker_corners: Vector<Vector<Point2f>>,
    marker_ids: Vector<i32>,
}

pub struct OpenCvCameraModel {
    pub opencv_camera: OpenCvCamera,
    pub on_calibration: bool,
    pub captured_frame_annotated_vec: Vec<FrameAnnotated>,
    pub camera_parameter: Option<CameraParameter>,
    pub camera_parameter_path: Option<String>,
}

impl OpenCvCameraModel {
    pub fn new(opencv_camera: OpenCvCamera) -> Self {
        Self {
            opencv_camera,
            on_calibration: false,
            captured_frame_annotated_vec: Vec::new(),
            camera_parameter: None,
            camera_parameter_path: None,
        }
    }

    pub fn get_latest_frame(&self) -> Mat {
        self.opencv_camera.get_latest_frame()
    }

    pub fn get_latest_charuco_markers(&self) -> CharucoMarker {
        self.opencv_camera.get_latest_charuco_markers()
    }

    pub fn capture_current_frame_with_annotated(&mut self) {
        let charuco_marker = self.get_latest_charuco_markers();
        self.captured_frame_annotated_vec.push(FrameAnnotated {
            frame: self.get_latest_frame(),
            charuco_corners: charuco_marker.charuco_corners,
            charuco_ids: charuco_marker.charuco_ids,
            marker_corners: charuco_marker.marker_corners,
            marker_ids: charuco_marker.marker_ids,
        });
    }

    pub fn clear_captured_frames_with_annotated(&mut self) {
        self.captured_frame_annotated_vec.clear();
    }

    pub fn calibrate_with_captured_frames(&mut self) -> Result<()> {
        let mut charuco_corners: Vector<Mat> = Vector::new();
        let mut charuco_ids: Vector<Mat> = Vector::new();

        for frame_annotated in &self.captured_frame_annotated_vec {
            charuco_corners.push(frame_annotated.charuco_corners.clone());
            charuco_ids.push(frame_annotated.charuco_ids.clone());
        }

        let image_size = self.opencv_camera.get_latest_frame().size().unwrap();

        let camera_parameter = self.opencv_camera.calibrate_camera(
            &mut charuco_corners,
            &mut charuco_ids,
            image_size,
        )?;

        self.camera_parameter = Some(camera_parameter);

        Ok(())
    }

    pub fn save_current_camera_parameter_to_file(
        &self,
        path: &str,
    ) -> Result<()> {
        if let Some(camera_parameter) = &self.camera_parameter {
            let camera_parameter_num: CameraParameterNum =
                camera_parameter.try_into()?;
            let serialized =
                serde_json::to_string(&camera_parameter_num).unwrap();
            std::fs::write(path, serialized).unwrap();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Camera parameter is not initialized"))
        }
    }

    pub fn load_camera_parameter_from_file(
        &mut self,
        path: &str,
    ) -> Result<()> {
        let serialized = std::fs::read_to_string(path)?;
        let camera_parameter_num: CameraParameterNum =
            serde_json::from_str(&serialized)?;
        self.camera_parameter = Some((&camera_parameter_num).try_into()?);
        Ok(())
    }
}

impl Clone for OpenCvCameraModel {
    fn clone(&self) -> Self {
        Self {
            opencv_camera: self.opencv_camera.clone(),
            on_calibration: self.on_calibration,
            captured_frame_annotated_vec: self
                .captured_frame_annotated_vec
                .clone(),
            camera_parameter: self.camera_parameter.clone(),
            camera_parameter_path: self.camera_parameter_path.clone(),
        }
    }
}
