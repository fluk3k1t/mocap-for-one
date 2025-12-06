use eframe::egui;
use egui_dock::DockState;
use egui_file::FileDialog;
use mocap_for_one::{CameraStream, VideoSourceConfig};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct UnityCameraModal {
    pub dialog: Option<FileDialog>,
    pub opened_path: Option<PathBuf>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct UnityCameraModalConfig {
    pub opened_path: Option<PathBuf>,
}

impl From<UnityCameraModalConfig> for UnityCameraModal {
    fn from(config: UnityCameraModalConfig) -> Self {
        Self {
            dialog: None,
            opened_path: config.opened_path,
        }
    }
}

impl From<&UnityCameraModal> for UnityCameraModalConfig {
    fn from(modal: &UnityCameraModal) -> Self {
        UnityCameraModalConfig {
            opened_path: modal.opened_path.clone(),
        }
    }
}

impl UnityCameraModal {
    pub fn new() -> Self {
        Self {
            dialog: None,
            opened_path: None,
        }
    }

    pub fn open(&mut self) {
        let mut d = FileDialog::open_file(self.opened_path.clone());
        d.open();
        self.dialog = Some(d);
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        camera_streams: &mut crate::widgets::CameraStreams,
        tree: &mut DockState<String>,
    ) {
        if let Some(d) = &mut self.dialog {
            d.show(ctx);
            if d.selected() {
                if let Some(path) = d.path() {
                    self.opened_path = Some(path.to_path_buf());
                    let config = VideoSourceConfig::MMAP {
                        path: path.to_string_lossy().into_owned(),
                    };

                    match CameraStream::new(config) {
                        Ok(stream) => {
                            let name = path
                                .file_name()
                                .and_then(OsStr::to_str)
                                .unwrap_or("unity_cam")
                                .to_string();
                            camera_streams.streams.insert(name.clone(), stream);
                            tree.push_to_focused_leaf(name);
                        }
                        Err(err) => {
                            eprintln!(
                                "Failed to open Unity camera source: {err}"
                            );
                        }
                    }
                }
            }
        }
    }
}
