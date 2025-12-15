mod app_state;
use app_state::{AppState, deserialize_app_state, serialize_app_state};
use eframe::egui;
use egui_tabs::Tabs;
use mocap_for_one::Workloads;
use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};

mod widgets;
use mocap_for_one::workload::WorkLoad;
use widgets::VideoCaptureModal;

use crate::widgets::{
    VideoViewer, unity_camera_modal::UnityCameraModalEffect,
    video_capture_modal::VideoCaptureModalEffect,
    video_viewer::VideoViewerEffect,
};

struct App {
    state: AppState,
    video_modal: VideoCaptureModal,
    status_message: Option<String>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.video_modal.show(ctx).map(|eff| match eff {
            VideoCaptureModalEffect::OnOpenCamera(cam) => {
                self.state.workload.add_camera_stream(cam);
            }
        });

        self.state.unity_modal.show(ctx).map(|eff| match eff {
            UnityCameraModalEffect::OnOpenCamera(cam) => {
                self.state.workload.add_camera_stream(cam);
            }
        });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.video_modal.open();
                }

                if ui.button("Add Unity Camera").clicked() {
                    self.state.unity_modal.open();
                }

                if ui.button("Save Config").clicked() {
                    self.status_message = match save_state_to_disk(&self.state)
                    {
                        Ok(()) => Some("Config saved.".to_owned()),
                        Err(err) => {
                            Some(format!("Failed to save config: {}", err))
                        }
                    };
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = Tabs::new(self.state.workload.opencv_cams.len() as i32)
                .show(ui, |ui, state| {
                    if let Some(opencv_cam) = self
                        .state
                        .workload
                        .opencv_cams
                        .get(state.index() as usize)
                    {
                        ui.add(
                            egui::Label::new(
                                opencv_cam.camera_stream_config.name.clone(),
                            )
                            .selectable(false),
                        );
                    }
                });

            if let Some(selected_tab) = tab.selected() {
                if let Some(selected_opencv_cam) =
                    self.state.workload.opencv_cams.get(selected_tab as usize)
                {
                    VideoViewer::new().show(ui, selected_opencv_cam).map(
                        |eff| match eff {
                            VideoViewerEffect::OnClose => {
                                self.state
                                    .workload
                                    .opencv_cams
                                    .remove(selected_tab as usize);
                            }
                        },
                    );
                }
            }
        });

        thread::sleep(Duration::from_millis(1000 / 60));
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(err) = save_state_to_disk(&self.state) {
            self.status_message =
                Some(format!("Failed to save config on exit: {}", err));
        }
    }
}

impl App {
    fn new() -> Result<Self> {
        let state = if let Ok(state) = load_state_from_disk() {
            state
        } else {
            AppState::default()
        };

        Ok(Self {
            state,
            video_modal: VideoCaptureModal::new(),
            status_message: None,
        })
    }
}

const CONFIG_PATH: &str = "mocap_for_one_config.json";

fn load_state_from_disk() -> Result<AppState> {
    match File::open(CONFIG_PATH) {
        Ok(file) => {
            let reader = BufReader::new(file);
            deserialize_app_state(reader)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            Ok(AppState::default())
        }
        Err(err) => {
            Err(anyhow::Error::new(err)).context("failed to open config file")
        }
    }
}

fn save_state_to_disk(state: &AppState) -> Result<()> {
    let file = File::create(CONFIG_PATH).with_context(|| {
        format!("failed to create config file '{}'", CONFIG_PATH)
    })?;
    let writer = BufWriter::new(file);
    serialize_app_state(state, writer).context("failed to write config")
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MocapForOne",
        options,
        Box::new(|_cc| {
            App::new()
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(Into::into)
        }),
    )
}
