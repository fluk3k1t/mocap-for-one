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

use crate::widgets::video_capture_modal::VideoCaptureModalEffect;

struct App {
    state: AppState,
    // tree: egui_dock::DockState<String>,
    video_modal: VideoCaptureModal,
    status_message: Option<String>,
    workload: WorkLoad,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // show camera open modal if requested
        self.video_modal.show(ctx).map(|eff| match eff {
            VideoCaptureModalEffect::OnOpenCamera(cam) => {
                self.workload.add_camera_stream(cam);
            }
        });
        // egui::CentralPanel::default().show(ctx, |ui| {

        // });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.video_modal.open();
                }

                // if ui.button("Add Unity Camera").clicked() {
                //     self.state.unity_modal.open();
                // }

                // if ui.button("Save Config").clicked() {
                //     self.status_message = match save_state_to_disk(&self.state)
                //     {
                //         Ok(()) => Some("Config saved.".to_owned()),
                //         Err(err) => {
                //             Some(format!("Failed to save config: {}", err))
                //         }
                //     };
                // }

                // if let Some(message) = &self.status_message {
                //     ui.label(message);
                // }
            });

            // Tabs::new(3).show(ui, |ui, state| {
            //     // ui.label(format!("{:?}", state.index()));
            //     ui.add(egui::Label::new("hello").selectable(false));
            // });
        });

        egui::CentralPanel::default().show(ctx, |ui_central| {
            let mut idx = 0;
            let tab = Tabs::new(self.workload.opencv_cams.len() as i32).show(
                ui_central,
                |ui, state| {
                    idx = state.index();
                    // ui.label(format!("{:?}", state.index()));
                    for (index, opencv_cam) in
                        self.workload.opencv_cams.iter().enumerate()
                    {
                        if (index as i32) == state.index() {
                            ui.add(
                                egui::Label::new(format!("{:?}", opencv_cam))
                                    .selectable(false),
                            );
                            // ui_central.label("jj");
                            // ui_central
                            //     .add(egui::Label::new("jj").selectable(false));
                            // egui::CentralPanel::default().show(ctx, |ui| {
                            //     ui.label("jj");
                            // });
                        }
                    }
                },
            );
            ui_central.separator();
            ui_central.label(format!("{}", idx));
            println!("{:?}", tab.selected());
        });

        // show unity file dialog if active
        // self.state.unity_modal.show(
        //     ctx,
        //     &mut self.state.camera_streams,
        //     &mut self.tree,
        // );

        // egui_dock::DockArea::new(&mut self.tree)
        //     .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
        //     .show(ctx, &mut self.state.camera_streams);

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
        let state = load_state_from_disk()?;
        // Reconstruct dock tree from loaded cameras
        // let mut tree = egui_dock::DockState::new(vec![]);
        // for name in state.camera_streams.streams.keys() {
        //     tree.push_to_focused_leaf(name.clone());
        // }

        Ok(Self {
            state,
            workload: WorkLoad::new(),
            // tree,
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
