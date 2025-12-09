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
    VideoViewer, video_capture_modal::VideoCaptureModalEffect,
};

struct App {
    state: AppState,
    // tree: egui_dock::DockState<String>,
    video_modal: VideoCaptureModal,
    status_message: Option<String>,
    // workload: WorkLoad,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // show camera open modal if requested
        self.video_modal.show(ctx).map(|eff| match eff {
            VideoCaptureModalEffect::OnOpenCamera(cam) => {
                self.state.workload.add_camera_stream(cam);
            }
        });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.video_modal.open();
                }

                // if ui.button("Add Unity Camera").clicked() {
                //     self.state.unity_modal.open();
                // }

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

            // Tabs::new(3).show(ui, |ui, state| {
            //     // ui.label(format!("{:?}", state.index()));
            //     ui.add(egui::Label::new("hello").selectable(false));
            // });
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
                            egui::Label::new(opencv_cam.stream.name.clone())
                                .selectable(false),
                        );
                    }
                });

            if let Some(selected_tab) = tab.selected() {
                if let Some(selected_opencv_cam) =
                    self.state.workload.opencv_cams.get(selected_tab as usize)
                {
                    VideoViewer::new().show(ui, selected_opencv_cam);
                }
            }

            // ui.separator();

            // let total_width = ui.available_width();
            //         let image_width = total_width * 0.8;
            //         let config_width = total_width * 0.2;

            //         let available_height = ui.available_height();
            //         ui.horizontal(|ui| {
            //             ui.allocate_ui_with_layout(
            //                 egui::Vec2::new(image_width, available_height),
            //                 egui::Layout::top_down(egui::Align::Center),
            //                 |ui| {
            //                     ui.centered_and_justified(|ui| {
            //                         if let Some(stream) = self.streams.get(tab) {
            //                             if let Some(img) = stream.latest_image() {
            //                                 let texture = ui.ctx().load_texture(
            //                                     format!("cam_frame_{}", tab),
            //                                     img.clone(),
            //                                     Default::default(),
            //                                 );

            //                                 let img_size = img.size;
            //                                 let available = egui::Vec2::new(
            //                                     image_width,
            //                                     available_height,
            //                                 );

            //                                 let scale_x = available.x / img_size[0] as f32;
            //                                 let scale_y = available.y / img_size[1] as f32;
            //                                 let scale = scale_x.min(scale_y).min(1.0);

            //                                 let display_size = egui::Vec2::new(
            //                                     img_size[0] as f32 * scale,
            //                                     img_size[1] as f32 * scale,
            //                                 );

            //                                 ui.image(egui::ImageSource::Texture(
            //                                     egui::load::SizedTexture::new(
            //                                         texture.id(),
            //                                         display_size,
            //                                     ),
            //                                 ));
            //                             } else {
            //                                 ui.centered_and_justified(|ui| {
            //                                     ui.label("No frame available yet");
            //                                 });
            //                             }
            //                         }
            //                     });
            //                 },
            //             );

            //             ui.separator();

            //             ui.allocate_ui_with_layout(
            //                 egui::Vec2::new(config_width, ui.available_height()),
            //                 egui::Layout::top_down(egui::Align::LEFT),
            //                 |ui| {
            //                     ui.heading("Camera Config");
            //                     ui.separator();

            //                     ui.label("Camera Settings:");
            //                     ui.add(
            //                         egui::Slider::new(&mut 50.0, 0.0..=100.0)
            //                             .text("Brightness"),
            //                     );
            //                     ui.add(
            //                         egui::Slider::new(&mut 50.0, 0.0..=100.0)
            //                             .text("Contrast"),
            //                     );
            //                     ui.add(
            //                         egui::Slider::new(&mut 50.0, 0.0..=100.0)
            //                             .text("Saturation"),
            //                     );
            //                     ui.add(
            //                         egui::Slider::new(&mut 30.0, 1.0..=60.0).text("FPS"),
            //                     );

            //                     ui.separator();

            //                     ui.label("Display Settings:");
            //                     ui.checkbox(&mut false, "Show crosshair");
            //                     ui.checkbox(&mut false, "Show grid");
            //                     ui.checkbox(&mut true, "Auto-fit image");

            //                     ui.separator();

            //                     ui.label("Recording:");
            //                     if ui.button("Start Recording").clicked() {
            //                         // TODO: Implement recording
            //                     }
            //                     if ui.button("Take Screenshot").clicked() {
            //                         // TODO: Implement screenshot
            //                     }

            //                     ui.separator();

            //                     ui.label(format!("Camera: {}", tab));
            //                     if let Some(stream) = self.streams.get(tab) {
            //                         if let Some(img) = stream.latest_image() {
            //                             let img_size = img.size;
            //                             ui.label(format!(
            //                                 "Resolution: {}x{}",
            //                                 img_size[0], img_size[1]
            //                             ));
            //                         }
            //                     }
            //                 },
            //             );
            //         });
            // ui.
            // ui_central.label(format!("{}", idx));
            // println!("{:?}", tab.selected());
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
        let state = if let Ok(state) = load_state_from_disk() {
            state
        } else {
            AppState::default()
        };
        // Reconstruct dock tree from loaded cameras
        // let mut tree = egui_dock::DockState::new(vec![]);
        // for name in state.camera_streams.streams.keys() {
        //     tree.push_to_focused_leaf(name.clone());
        // }

        Ok(Self {
            state,
            // workload: WorkLoad::new(),
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
