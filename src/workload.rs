use serde::{Deserialize, Serialize};

use crate::{CameraStream, VideoSourceConfig, camera_stream};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkLoadConfig {
    pub opencv_cams: Vec<OpenCvCameraConfig>,
}

pub struct WorkLoad {
    pub opencv_cams: Vec<OpenCvCamera>,
}

impl TryFrom<WorkLoadConfig> for WorkLoad {
    type Error = anyhow::Error;

    fn try_from(config: WorkLoadConfig) -> Result<Self, Self::Error> {
        let opencv_cams = config
            .opencv_cams
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { opencv_cams })
    }
}

impl From<&WorkLoad> for WorkLoadConfig {
    fn from(workload: &WorkLoad) -> Self {
        Self {
            opencv_cams: workload.opencv_cams.iter().map(Into::into).collect(),
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
        self.opencv_cams.push(OpenCvCamera::new(stream));
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenCvCameraConfig {
    pub name: String,
    pub video_source_config: VideoSourceConfig,
}

#[derive(Debug)]
pub struct OpenCvCamera {
    pub stream: CameraStream,
}

impl TryFrom<OpenCvCameraConfig> for OpenCvCamera {
    type Error = anyhow::Error;

    fn try_from(config: OpenCvCameraConfig) -> Result<Self, Self::Error> {
        let stream = CameraStream::new(
            config.name,
            config.video_source_config.try_into()?,
        )?;
        Ok(Self { stream })
    }
}

impl From<&OpenCvCamera> for OpenCvCameraConfig {
    fn from(camera: &OpenCvCamera) -> Self {
        Self {
            name: camera.stream.name.clone(),
            video_source_config: camera.stream.video_source_config.clone(),
        }
    }
}

impl OpenCvCamera {
    pub fn new(stream: CameraStream) -> Self {
        Self { stream }
    }
}
