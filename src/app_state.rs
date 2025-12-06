use crate::widgets::{
    CameraStreams, CameraStreamsConfig, UnityCameraModal,
    UnityCameraModalConfig,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub unity_camera_modal: UnityCameraModalConfig,
    #[serde(alias = "tcam_buffers")]
    pub camera_streams: CameraStreamsConfig,
}

pub struct AppState {
    pub camera_streams: CameraStreams,
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
        let camera_streams = CameraStreams::try_from(config.camera_streams)?;
        Ok(Self {
            camera_streams,
            unity_modal: config.unity_camera_modal.into(),
        })
    }
}

impl From<&AppState> for AppConfig {
    fn from(state: &AppState) -> Self {
        Self {
            camera_streams: (&state.camera_streams).into(),
            unity_camera_modal: (&state.unity_modal).into(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            camera_streams: CameraStreams::new(),
            unity_modal: UnityCameraModal::new(),
        }
    }
}
