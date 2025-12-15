use serde::{Deserialize, Serialize};

use crate::{
    CameraStream, OpenCvCamera, OpenCvCameraConfig, VideoSourceConfig,
    camera_stream,
};

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
