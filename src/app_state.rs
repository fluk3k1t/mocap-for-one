use crate::widgets::{UnityCameraModal, UnityCameraModalConfig};
use anyhow::{Context, Result};
use mocap_for_one::{VideoSource, WorkLoad, WorkLoadConfig};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Serialize, Deserialize)]
// #[serde(default)]
pub struct AppConfig {
    pub unity_camera_modal: UnityCameraModalConfig,
    pub workload: WorkLoadConfig,
}

pub struct AppState {
    pub workload: WorkLoad,
    pub unity_modal: UnityCameraModal,
}

pub fn serialize_app_state<W: Write>(
    state: &AppState,
    mut writer: W,
) -> Result<()> {
    let config: AppConfig = state.into();
    serde_json::to_writer_pretty(&mut writer, &config)
        .context("failed to serialize config")?;
    writer.flush().context("failed to flush config writer")?;
    Ok(())
}

pub fn deserialize_app_state<R: Read>(reader: R) -> Result<AppState> {
    let config: AppConfig =
        serde_json::from_reader(reader).context("failed to parse config")?;
    AppState::try_from(config)
        .context("failed to convert config into app state")
}

impl TryFrom<AppConfig> for AppState {
    type Error = anyhow::Error;

    fn try_from(config: AppConfig) -> Result<Self, Self::Error> {
        // let camera_streams = CameraStreams::try_from(config.camera_streams)?;
        Ok(Self {
            // camera_streams,
            workload: config.workload.try_into()?,
            unity_modal: config.unity_camera_modal.into(),
        })
    }
}

impl From<&AppState> for AppConfig {
    fn from(state: &AppState) -> Self {
        Self {
            workload: (&state.workload).into(),
            unity_camera_modal: (&state.unity_modal).into(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            workload: WorkLoad::new(),
            unity_modal: UnityCameraModal::new(),
        }
    }
}
