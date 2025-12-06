use eframe::egui::{self, Context, Id, Modal};
use egui_dock::DockState;
use mocap_for_one::{CameraStream, VideoSourceConfig, enumerate_cameras};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoCaptureModal {
    pub open: bool,
    pub enum_cameras: Vec<(i32, String)>,
    pub selected: Option<(i32, String)>,
}

impl VideoCaptureModal {
    pub fn new() -> Self {
        Self {
            open: false,
            enum_cameras: vec![],
            selected: None,
        }
    }

    pub fn open(&mut self) {
        if let Ok(list) = enumerate_cameras() {
            // println!()
            self.enum_cameras = list;
        }
        self.open = true;
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        tcam_buffers: &mut crate::widgets::CameraStreams,
        tree: &mut DockState<String>,
    ) {
        if !self.open {
            return;
        }

        let _ = Modal::new(Id::new("Open Camera Modal")).show(ctx, |ui| {
            ui.heading("Select Camera Device");

            egui::ComboBox::from_label("")
                .selected_text(format!(
                    "{}",
                    self.selected
                        .as_ref()
                        .map(|(_, name)| name)
                        .unwrap_or(&"none".to_string()),
                ))
                .show_ui(ui, |ui| {
                    for (index, name) in &self.enum_cameras {
                        ui.selectable_value(
                            &mut self.selected,
                            Some((*index, name.clone())),
                            format!("{}", name),
                        );
                    }
                });

            ui.separator();

            egui::Sides::new().show(
                ui,
                |_left_ui| {},
                |ui| {
                    if ui.button("Open").clicked() {
                        if let Some((index, name)) = &self.selected {
                            let config =
                                VideoSourceConfig::Capture { index: *index };

                            match CameraStream::new(config) {
                                Ok(tcam) => {
                                    tcam_buffers
                                        .streams
                                        .insert(name.clone(), tcam);
                                    tree.push_to_focused_leaf(name.clone());
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Failed to open camera {name}: {err}"
                                    );
                                }
                            }

                            self.open = false;
                            self.selected = None;
                        }
                    }
                },
            );
        });
    }
}
