use crate::{CameraStream, camera_stream};

pub struct WorkLoad {
    pub opencv_cams: Vec<OpenCvCamera>,
}

impl WorkLoad {
    pub fn new() -> Self {
        Self {
            opencv_cams: vec![],
        }
    }

    pub fn add_camera_stream(&mut self, stream: CameraStream) {
        self.opencv_cams.push(OpenCvCamera::new(stream));
    }
}

#[derive(Debug)]
pub struct OpenCvCamera {
    camera_stream: CameraStream,
}

impl OpenCvCamera {
    pub fn new(camera_stream: CameraStream) -> Self {
        Self { camera_stream }
    }
}
