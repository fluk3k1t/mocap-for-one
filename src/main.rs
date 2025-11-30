use chrono::format;
use eframe::{
    egui::{self, Id, Modal, Window},
    epaint::image,
};
use egui_file::FileDialog;
use mocap_for_one::{MMAPCam, ThreadingCam, enumerate_cameras};
use opencv::{
    core::{Mat, MatTraitConst, MatTraitConstManual},
    objdetect::{CharucoBoard, CharucoDetector},
    videoio::{CAP_ANY, VideoCapture, VideoCaptureTrait},
};
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, thread, time::Duration};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

struct TCamBuffers {
    tcams: BTreeMap<String, ThreadingCam>,
}

impl egui_dock::TabViewer for TCamBuffers {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Left side: Camera image (70% of width)
        let total_width = ui.available_width();
        let image_width = total_width * 0.8;
        let config_width = total_width * 0.2;

        let available_height = ui.available_height();
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::Vec2::new(image_width, available_height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        if let Some(tcam) = self.tcams.get(tab) {
                            if let Some(img) = tcam.get_latest_image() {
                                let cd = CharucoDetector::new_def(
                                    &CharucoBoard::default().unwrap(),
                                );

                                let texture = ui.ctx().load_texture(
                                    format!("cam_frame_{}", tab),
                                    img.clone(),
                                    Default::default(),
                                );

                                // Calculate scaled size to fit the available area while maintaining aspect ratio
                                let available_size = ui.available_size();
                                let img_size = img.size;

                                let scale_x =
                                    available_size.x / img_size[0] as f32;
                                let scale_y =
                                    available_size.y / img_size[1] as f32;
                                let scale = scale_x.min(scale_y);

                                let display_size = egui::Vec2::new(
                                    img_size[0] as f32 * scale,
                                    img_size[1] as f32 * scale,
                                );

                                ui.image(egui::ImageSource::Texture(
                                    egui::load::SizedTexture::new(
                                        texture.id(),
                                        display_size,
                                    ),
                                ));
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No frame available yet");
                                });
                            }
                        }
                    });
                },
            );

            ui.separator();

            // Right side: Configuration panel (30% of width)
            ui.allocate_ui_with_layout(
                egui::Vec2::new(config_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.heading("Camera Config");
                    ui.separator();

                    ui.label("Camera Settings:");
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Brightness"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Contrast"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Saturation"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 30.0, 1.0..=60.0).text("FPS"),
                    );

                    ui.separator();

                    ui.label("Display Settings:");
                    ui.checkbox(&mut false, "Show crosshair");
                    ui.checkbox(&mut false, "Show grid");
                    ui.checkbox(&mut true, "Auto-fit image");

                    ui.separator();

                    ui.label("Recording:");
                    if ui.button("Start Recording").clicked() {
                        // TODO: Implement recording
                    }
                    if ui.button("Take Screenshot").clicked() {
                        // TODO: Implement screenshot
                    }

                    ui.separator();

                    ui.label(format!("Camera: {}", tab));
                    if let Some(tcam) = self.tcams.get(tab) {
                        if let Some(img) = tcam.get_latest_image() {
                            let img_size = img.size;
                            ui.label(format!(
                                "Resolution: {}x{}",
                                img_size[0], img_size[1]
                            ));
                        }
                    }
                },
            );
        });
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

struct App {
    tcam_buffers: TCamBuffers,
    tree: egui_dock::DockState<String>,
    open_camera_modal_is_open: bool,
    open_unity_camera_modal_is_open: bool,
    opened_unity_camera_path: Option<PathBuf>,
    open_unity_camera_file_dialog: Option<FileDialog>,
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

        if self.open_camera_modal_is_open {}

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.enum_cameras = enumerate_cameras()
                        .expect("Failed to enumerate cameras");
                    self.open_camera_modal_is_open = true;
                }

                if ui.button("Add Unity Camera").clicked() {
                    let mut dialog = FileDialog::open_file(
                        self.opened_unity_camera_path.clone(),
                    );
                    dialog.open();
                    self.open_unity_camera_file_dialog = Some(dialog);
                    // self.open_unity_camera_modal_is_open = true;
                }
                // if ui
                // ui.color_edit_button_rgb(&mut [0.0, 255.0, 0.0])
                // if ui.button("Run").clicked() {}

                // if ui.button("Save").clicked() {}
                // {}
            });
        });

        if let Some(dialog) = &mut self.open_unity_camera_file_dialog {
            dialog.show(ctx);
            if dialog.selected() {
                if let Some(path) = dialog.path() {
                    self.opened_unity_camera_path = Some(path.to_path_buf());

                    let unity_cam = MMAPCam::new(path).unwrap();
                    let tcam = ThreadingCam::new(unity_cam);
                    self.tcam_buffers.tcams.insert(
                        path.file_name()
                            .and_then(OsStr::to_str)
                            .unwrap_or("unity_cam")
                            .to_string(),
                        tcam,
                    );
                    self.tree.push_to_focused_leaf(
                        path.file_name()
                            .and_then(OsStr::to_str)
                            .unwrap_or("unity_cam")
                            .to_string(),
                    );
                }
            }
        }

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tcam_buffers);

        thread::sleep(Duration::from_millis(1000 / 60));
        ctx.request_repaint();
    }
}

impl App {
    fn new() -> Self {
        let tree = egui_dock::DockState::new(vec![]);

        Self {
            tcam_buffers: TCamBuffers {
                tcams: BTreeMap::new(),
            },
            tree,
            open_camera_modal_is_open: false,
            open_unity_camera_modal_is_open: false,
            opened_unity_camera_path: None,
            open_unity_camera_file_dialog: None,
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
