use chrono::format;
use eframe::egui::{self, Id, Modal, Window};
use mocap_for_one::{ThreadingCam, enumerate_cameras};
use opencv::{
    core::{Mat, MatTraitConst, MatTraitConstManual},
    videoio::{CAP_ANY, VideoCapture, VideoCaptureTrait},
};
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, thread, time::Duration};

struct TCamBuffers {
    tcams: BTreeMap<String, ThreadingCam>,
}

impl egui_dock::TabViewer for TCamBuffers {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if let Some(tcam) = self.tcams.get(tab) {
            if let Some(img) = tcam.get_latest_image() {
                let texture = ui.ctx().load_texture(
                    format!("cam_frame_{}", tab),
                    img,
                    Default::default(),
                );
                ui.image(&texture);
            } else {
                ui.label("No frame available yet");
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

struct App {
    tcam_buffers: TCamBuffers,
    tree: egui_dock::DockState<String>,
    open_camera_modal_is_open: bool,
    enum_cameras: Vec<(i32, String)>,
    selected_camera: Option<(i32, String)>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.open_camera_modal_is_open {
            let open_camera_modal = Modal::new(Id::new("Open Camera Modal"))
                .show(ctx, |ui| {
                    ui.heading("Select Camera Device");

                    egui::ComboBox::from_label("")
                        .selected_text(format!(
                            "{}",
                            self.selected_camera
                                .as_ref()
                                .map(|(_, name)| name)
                                .unwrap_or(&"none".to_string()),
                        ))
                        .show_ui(ui, |ui| {
                            for (index, name) in &self.enum_cameras {
                                ui.selectable_value(
                                    &mut self.selected_camera,
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
                                if let Some((index, name)) =
                                    &self.selected_camera
                                {
                                    let vcap =
                                        VideoCapture::new(*index, CAP_ANY)
                                            .unwrap();
                                    let tcam = ThreadingCam::new(vcap);
                                    self.tcam_buffers
                                        .tcams
                                        .insert(name.clone(), tcam);
                                    self.tree
                                        .push_to_focused_leaf(name.clone());
                                    self.open_camera_modal_is_open = false;
                                    self.selected_camera = None;
                                }
                            }
                        },
                    );
                });
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.enum_cameras = enumerate_cameras()
                        .expect("Failed to enumerate cameras");
                    self.open_camera_modal_is_open = true;
                }
            });
        });

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tcam_buffers);

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     let img_lock = self.img.lock().unwrap();
        //     if let Some(ref img) = *img_lock {
        //         let texture = ui.ctx().load_texture(
        //             "cam_frame",
        //             img.clone(),
        //             Default::default(),
        //         );
        //         ui.image(&texture);
        //         println!("Rendered frame");
        //     } else {
        //         println!("No frame available yet");
        //     }

        thread::sleep(Duration::from_millis(1000 / 60));
        // });

        ctx.request_repaint();
    }
}

impl App {
    fn new() -> Self {
        let tree = egui_dock::DockState::new(vec![]);

        // let mut tcams = BTreeMap::new();
        // tcams.

        Self {
            tcam_buffers: TCamBuffers {
                tcams: BTreeMap::new(),
            },
            tree,
            open_camera_modal_is_open: false,
            enum_cameras: vec![],
            selected_camera: None,
        }
    }

    fn render_cam(
        &mut self,
        ui: &mut egui::Ui,
    ) -> opencv::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
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
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
