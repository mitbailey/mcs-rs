#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::any::Any;
use std::vec;

use eframe::egui;
use eframe::egui::{Visuals, Margin};
use egui::menu;
use egui::{Frame, Widget};
use serialport::SerialPortInfo;
use egui_dock::{DockArea, DockState, NodeIndex};

pub mod drivers;
pub mod middleware;
use middleware::{MotionController, MovementAxesIndices, Detector};

// use rand::prelude::*;

use crate::middleware::DetectorMiddleware;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        // .with_icon(
        //     // NOTE: Adding an icon is optional
        //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
        //         .expect("Failed to load icon"),
        // ),
        ..Default::default()
    };

    eframe::run_native(
        "MCS",
        native_options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<Mcs>::default()
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| {
                    // This gives us image support:
                    egui_extras::install_image_loaders(&cc.egui_ctx);

                    Box::<Mcs>::default()
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

enum ActivePage {
    DeviceManager,
    MainWindow,
    MachineConfig,
}

enum DialogType {
    Debug,
    Info,
    Warn,
    Error,
}

impl DialogType {
    fn as_str(&self) -> &str {
        match self {
            DialogType::Debug => "DEBUG",
            DialogType::Info => "INFO",
            DialogType::Warn => "WARN",
            DialogType::Error => "ERROR",
        }
    }
}

// enum ActivePopUp {
//     DeviceManager,
// }

struct McsTabs {
    modal_active: bool,

    num_mc_devs: usize,
    num_det_devs: usize,
    sel_mc_port: Vec<String>,
    sel_det_port: Vec<String>,
    sel_mc_model: Vec<String>,
    sel_det_model: Vec<String>,
    sel_mc_nick: Vec<String>,
    sel_det_nick: Vec<String>,

    // Controls
    pos_target: f32,
    pos_curr: f32,
    scan_start: f32,
    scan_end: f32,
    scan_step: f32,
    scan_repeats: u32,
    samp_rot_target: f32,
    samp_rot_curr: f32,
    samp_ang_target: f32,
    samp_ang_curr: f32,
    samp_tran_target: f32,
    samp_tran_curr: f32,
    samp_scan_type: String,
    samp_scan_start: f32,
    samp_scan_end: f32,
    samp_scan_step: f32,
    samp_scan_repeats: u32,

    detector_data: Vec<Vec<f64>>, // Outer vec is per-detector, inner vec is per-scan data.

    connd_detectors: Vec<Detector>,
}

impl egui_dock::TabViewer for McsTabs {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.set_enabled(!self.modal_active);

        match tab.as_str() {
            "Device Controls" => self.device_controls(ui),
            "Data Plot" => self.data_plot(ui),
            "Data Log" => self.data_log(ui),
            _ => self.default_tab(ui),
        }
    }
}

impl McsTabs {
    fn device_controls(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::CollapsingHeader::new("Main Drive").show(ui, |ui| {
                ui.label("Manual Control");
                ui.horizontal(|ui| {
                    ui.button("Home");
                    ui.label("Position [nm]");
                    ui.add(egui::DragValue::new(&mut self.pos_target).speed(0.1));
                    ui.button("Move");
                    ui.label(format!("{} nm", self.pos_curr));
                });

                ui.separator();

                ui.label("Scanning Control");
                ui.horizontal(|ui| {
                    ui.label("Start [nm]");
                    ui.add(egui::DragValue::new(&mut self.scan_start).speed(0.1));
                    ui.label("End [nm]");
                    ui.add(egui::DragValue::new(&mut self.scan_end).speed(0.1));
                    ui.label("Step [nm]");
                    ui.add(egui::DragValue::new(&mut self.scan_step).speed(0.1));
                    ui.label("Repeats");
                    ui.add(egui::DragValue::new(&mut self.scan_repeats).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.button("Start");
                    ui.button("Pause");
                    ui.button("Stop");
                });

                ui.button("Scan");

            });
            egui::CollapsingHeader::new("Filter Wheel").show(ui, |ui| {
                ui.label("Body");
            });
            egui::CollapsingHeader::new("Sample").show(ui, |ui| {
                ui.label("Manual Control");
                ui.horizontal(|ui| {
                    ui.button("Home");
                    ui.label("Rotation [deg]");
                    ui.add(egui::DragValue::new(&mut self.samp_rot_target).speed(0.1));
                    ui.button("Move");
                    ui.label(format!("{} deg", self.samp_rot_curr));
                });
                ui.horizontal(|ui| {
                    ui.button("Home");
                    ui.label("Angle [deg]");
                    ui.add(egui::DragValue::new(&mut self.samp_ang_target).speed(0.1));
                    ui.button("Move");
                    ui.label(format!("{} deg", self.samp_ang_curr));
                });
                ui.horizontal(|ui| {
                    ui.button("Home");
                    ui.label("Translation [nm]");
                    ui.add(egui::DragValue::new(&mut self.samp_tran_target).speed(0.1));
                    ui.button("Move");
                    ui.label(format!("{} nm", self.samp_tran_curr));
                });

                ui.label("Scanning Control");
                ui.horizontal(|ui| {
                    ui.label("Scan Type");
                    egui::ComboBox::from_id_source("Sample scan Type")
                        .selected_text(format!("{:?}", self.samp_scan_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.samp_scan_type, "Rotation".to_string(), "Rotation");
                            ui.selectable_value(&mut self.samp_scan_type, "Translation".to_string(), "Translation");
                            ui.selectable_value(&mut self.samp_scan_type, "Theta to Theta".to_string(), "Theta to Theta");
                        }
                    );
                });
                
                ui.horizontal(|ui| {
                    ui.label("Start [nm]");
                    ui.add(egui::DragValue::new(&mut self.samp_scan_start).speed(0.1));
                    ui.label("End [nm]");
                    ui.add(egui::DragValue::new(&mut self.samp_scan_end).speed(0.1));
                    ui.label("Step [nm]");
                    ui.add(egui::DragValue::new(&mut self.samp_scan_step).speed(0.1));
                    ui.label("Repeats");
                    ui.add(egui::DragValue::new(&mut self.samp_scan_repeats).speed(0.1));
                });
                
                ui.horizontal(|ui| {
                    ui.button("Start");
                    ui.button("Pause");
                    ui.button("Stop");
                });
            });
            egui::CollapsingHeader::new("Detector").show(ui, |ui| {
                ui.label("Body");
            });
        });
    }

    // TODO: Make a data_record function.

    fn data_plot(&mut self, ui: &mut egui::Ui) {
        use egui_plot::{Line, PlotPoints};
        let n = 128;
        
        let line = Line::new(
            (0..=n)
                .map(|i| {
                    use std::f64::consts::TAU;
                    let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
                    [x, 10.0 * gaussian(x)]
                })
                .collect::<PlotPoints>(),
        );
        
        let plot = egui_plot::Plot::new("test_plot")
            .legend(egui_plot::Legend::default())
            .y_axis_label("Photocurrent [pA]")
            .x_axis_label("Wavelength [nm]");

        plot.show(ui, |plot_ui| {
            for i in 0..self.detector_data.len() {
                // TODO: Some sort of show/dont show condition.

                plot_ui.line(Line::new(
                    self.detector_data[i]
                        .iter()
                        .enumerate()
                        .map(|(i, &y)| [i as f64, y])
                        .collect::<PlotPoints>(),
                ));

                // Line::new(self.detector_data)
                // plot_ui.line(&line);
            }
        });

        // TODO: Have this button also push to the data record as well.
        // TODO: Remove (test for the plot).
        // TEST: Button that generates random data one float at a time.
        ui.vertical(|ui| {
            if ui.button("Generate Random Datapoint").clicked() {
                for i in 0..self.detector_data.len() {
                    self.detector_data[i].push(self.connd_detectors[i].detect());
                    // TODO: Repaint done when scan is active... use scan_active bool or something.
                    ui.ctx().request_repaint();
                }
            }
        });
    }

    fn data_log(&mut self, ui: &mut egui::Ui) {
        ui.label("This is the data log tab.");
    }

    fn default_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("This is a default tab.");
    }
}

struct Mcs {
    // ports: String,
    devs_setup: bool,
    dark_mode: bool,

    dialog_type: DialogType,
    modal_message: String,
    active_page: ActivePage,

    search_ports: bool,
    ports: Result<Vec<SerialPortInfo>, serialport::Error>,

    tabs: McsTabs,
    tree: DockState<String>,

    devices_loading_progress: f32,
    devices_loading: bool,

    // TODO: Move to a middleware
    mtn_ctrl_models: Vec<String>, // Supported models
    det_models: Vec<String>, // Supported models

    first_time: bool,
    connd_mtn_ctrlrs: Vec<MotionController>,

    mai: MovementAxesIndices,


}

impl Default for Mcs {
    fn default() -> Self {
        let mut tree = DockState::new(vec!["Device Controls".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.main_surface_mut().split_right(
            NodeIndex::root(),
            0.5,
            vec!["Data Plot".to_owned()],
        );
        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.8, vec!["Data Log".to_owned()]);

        let tabs = McsTabs {
            modal_active: false,

            num_mc_devs: 1,
            num_det_devs: 1,

            sel_mc_port: Vec::new(),
            sel_det_port: Vec::new(),

            sel_mc_model: Vec::new(),
            sel_det_model: Vec::new(),

            sel_mc_nick: Vec::new(),
            sel_det_nick: Vec::new(),

            pos_target: 0.0,
            pos_curr: 0.0,
            scan_start: 0.0,
            scan_end: 0.0,
            scan_step: 0.0,
            scan_repeats: 0,
            samp_rot_target: 0.0,
            samp_rot_curr: 0.0,
            samp_ang_target: 0.0,
            samp_ang_curr: 0.0,
            samp_tran_target: 0.0,
            samp_tran_curr: 0.0,
            samp_scan_type: "None".to_owned(),
            samp_scan_start: 0.0,
            samp_scan_end: 0.0,
            samp_scan_step: 0.0,
            samp_scan_repeats: 0,

            detector_data: Vec::new(),

            connd_detectors: Vec::new(),
        };

        Self {
            // ports: String::new(),
            devs_setup: false,
            dark_mode: false,

            dialog_type: DialogType::Debug,
            modal_message: String::new(),
            active_page: ActivePage::DeviceManager,

            search_ports: true,
            ports: Ok(Vec::new()),

            devices_loading_progress: 0.0,
            devices_loading: false,

            tabs,
            tree,
            
            mtn_ctrl_models: vec!["MC1".to_owned(), "MC2".to_owned(), "MC3".to_owned()],
            det_models: vec!["De1".to_owned(), "De2".to_owned(), "De3".to_owned()],

            first_time: true,
            connd_mtn_ctrlrs: Vec::new(),
            // connd_detectors: Vec::new(),
            mai: MovementAxesIndices::default(),


        }
    }
}

impl Mcs {
    /// Instantiates an instance of a modal dialog window.
    fn dialog(&mut self, dialog_type: DialogType, message: &str) {
        match self.tabs.modal_active {
            true => {
                println!(
                    "A modal window is already active. The offending request was: [{}] {}",
                    dialog_type.as_str(),
                    message
                );
            }
            false => {
                self.tabs.modal_active = true;
                self.dialog_type = dialog_type;
                self.modal_message = message.to_owned();
            }
        }
    }

    /// Should be called each frame a dialog window needs to be shown.
    ///
    /// Should not be used to instantiate an instance of a dialog window, use `dialog()` instead.
    fn show_dialog(&mut self, ctx: &egui::Context) {
        self.tabs.modal_active = true;

        let title = self.dialog_type.as_str();

        egui::Window::new(title)
            .collapsible(false)
            .open(&mut self.tabs.modal_active)
            .show(ctx, |ui| {

                ui.horizontal(|ui| {
                    let scale = 0.25;
                    match self.dialog_type {
                        DialogType::Debug => {
                            ui.add(egui::Image::new(egui::include_image!("../res/Mcs_Information.png")).fit_to_original_size(scale));
                        }
                        DialogType::Info => {
                            ui.add(egui::Image::new(egui::include_image!("../res/Mcs_Information.png")).fit_to_original_size(scale));
                        }
                        DialogType::Warn => {
                            ui.add(egui::Image::new(egui::include_image!("../res/Mcs_Warning.png")).fit_to_original_size(scale));
                        }
                        DialogType::Error => {
                            ui.add(egui::Image::new(egui::include_image!("../res/Mcs_Error.png")).fit_to_original_size(scale));
                        }
                    }
                    // ui.add(egui::Image::new(egui::include_image!(img_path)));
                    // ui.add(egui::Image::new(egui::include_image!(self.dialog_type.get_image_url())));

                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(self.modal_message.to_owned()).wrap(true));
                    });
                });

                // if ui.button("Ok").clicked() {
                //     self.modal_active = false;
                // }

            });
    }
}

fn gaussian(x: f64) -> f64 {
    let var: f64 = 2.0;
    f64::exp(-(x / var).powi(2)) / (var * f64::sqrt(std::f64::consts::TAU))
}

impl McsTabs {
    fn nested_menus(ui: &mut egui::Ui) {
        if ui.button("Open...").clicked() {
            ui.close_menu();
        }
        ui.menu_button("SubMenu", |ui| {
            ui.menu_button("SubMenu", |ui| {
                if ui.button("Open...").clicked() {
                    ui.close_menu();
                }
                let _ = ui.button("Item");
            });
            ui.menu_button("SubMenu", |ui| {
                if ui.button("Open...").clicked() {
                    ui.close_menu();
                }
                let _ = ui.button("Item");
            });
            let _ = ui.button("Item");
            if ui.button("Open...").clicked() {
                ui.close_menu();
            }
        });
        ui.menu_button("SubMenu", |ui| {
            let _ = ui.button("Item1");
            let _ = ui.button("Item2");
            let _ = ui.button("Item3");
            let _ = ui.button("Item4");
            if ui.button("Open...").clicked() {
                ui.close_menu();
            }
        });
        let _ = ui.button("Very long text for this item");
    }
}

impl eframe::App for Mcs {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);


        // // Test of the MotionController indexing system.
        // if self.first_time {
        //     let mc1 = MotionController::new(11, Box::new(drivers::mp_789a_4::Mp789a4::new("COM1".to_string()).unwrap()));
        //     // let mc2 = MotionController::new(22);
        //     // let mc3 = MotionController::new(33);

        //     self.connd_mtn_ctrlrs.push(mc1);
        //     // self.connd_mtn_ctrlrs.push(mc2);
        //     // self.connd_mtn_ctrlrs.push(mc3);

        //     let dt1 = Detector::new(1, Box::new(drivers::ki_6485::Ki6485::new("COM2".to_string(), 10)));
            
        //     self.connd_detectors.push(dt1);

        //     self.mai.md_idx = Some(0);

        //     println!("ID: {}", self.connd_mtn_ctrlrs[self.mai.md_idx.expect("No idx.")].id);

        //     // self.mai.md_idx = Some(1);

        //     // println!("ID: {}", self.connd_mtn_ctrlrs[self.mai.md_idx.expect("No idx.")].id);

        //     // self.mai.md_idx = Some(2);

        //     // println!("ID: {}", self.connd_mtn_ctrlrs[self.mai.md_idx.expect("No idx.")].id);

        //     self.first_time = false;
        // }

        //////////////////////////////////////////////////////////////
        // All possible modal window popups should be handled here. //
        //////////////////////////////////////////////////////////////

        // There should only ever be one modal window active, and it should be akin to a dialog window - info, warn, or error.

        if self.tabs.modal_active {
            self.show_dialog(ctx);
        }

        //////////////////////////////////////////////////////////////
        //////////////////////////////////////////////////////////////
        //////////////////////////////////////////////////////////////

        // Debug Controls Window for Developer Use Only
        egui::Window::new("Developer Controls").show(ctx, |ui| {
            // ui.heading("Developer Controls");
            ui.horizontal(|ui| {
                ui.label("Modal Controller:");
                if ui.button("Close").clicked() {
                    self.tabs.modal_active = false;
                }
                if ui.button("Debug").clicked() {
                    self.dialog(DialogType::Debug, "This is a debug message. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam pharetra ex quis lacus efficitur luctus. Praesent sed lectus convallis, malesuada ex nec, pulvinar tortor. Pellentesque suscipit malesuada diam, sit amet lacinia nisi maximus in. Praesent mi tortor, pulvinar et pretium sed, maximus vitae nulla. Sed vitae nibh a ligula tempus rhoncus et ac mauris. Proin ipsum eros, aliquet quis sodales ac, egestas in mi. Curabitur est metus, sollicitudin in tincidunt ut, pulvinar eget turpis. Cras nec mattis quam, non ornare ipsum. Aliquam et viverra mauris, eget semper metus. Morbi imperdiet dui est, id posuere leo luctus imperdiet. ");
                }
                if ui.button("Info").clicked() {
                    self.dialog(DialogType::Info, "This is an informational message. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam pharetra ex quis lacus efficitur luctus. Praesent sed lectus convallis, malesuada ex nec, pulvinar tortor. Pellentesque suscipit malesuada diam, sit amet lacinia nisi maximus in. Praesent mi tortor, pulvinar et pretium sed, maximus vitae nulla. Sed vitae nibh a lig");
                }
                if ui.button("Warn").clicked() {
                    self.dialog(DialogType::Warn, "This is a warning message. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam pharetra ex quis lacus efficitur luctus. Praesent sed lectus convallis, malesuada ex nec, pulvinar tortor. Pe");
                }
                if ui.button("Error").clicked() {
                    self.dialog(DialogType::Error, "This is an error message.");
                }
            });
        });

        // Top Settings Panel
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_enabled(!self.tabs.modal_active);

                ui.horizontal(|ui| {
                    menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {
                                // …
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Open").clicked() {
                                // …
                            }
                        });
                        ui.menu_button("View", |ui| match self.dark_mode {
                            true => {
                                if ui.button("Switch to Light Mode").clicked() {
                                    ctx.set_visuals(Visuals::light());
                                    self.dark_mode = false;
                                }
                            }
                            false => {
                                if ui.button("Switch to Dark Mode").clicked() {
                                    ctx.set_visuals(Visuals::dark());
                                    self.dark_mode = true;
                                }
                            }
                        });
                        ui.menu_button("About", |ui| {
                            if ui.button("Open").clicked() {
                                // …
                            }
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("Open").clicked() {
                                // …
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label(format!("v{}", env!("CARGO_PKG_VERSION")))
                    });
                });
            });

        // Side Navigation Panel
        let mut left_panel_frame = Frame::default();
        left_panel_frame = left_panel_frame.inner_margin(Margin::ZERO);
        if self.dark_mode {
            left_panel_frame = left_panel_frame.fill(egui::style::Visuals::dark().window_fill);
        } else {
            left_panel_frame = left_panel_frame.fill(egui::style::Visuals::light().window_fill);
        }

        egui::SidePanel::left("side_panel")
            .frame(left_panel_frame)
            .resizable(false)
            .default_width(50.0)
            .max_width(50.0)
            .show(ctx, |ui| {
                ui.set_enabled(!self.tabs.modal_active);

                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
                ui.style_mut().spacing.button_padding = egui::vec2(0.0, 0.0);
                ui.style_mut().spacing.window_margin = egui::Margin::ZERO;

                // ui.heading("Navigation");

                // Load an image from a file and show it:
                let nav_dev_man =
                    egui::ImageButton::new(egui::include_image!("../res/mcs_devman_lite_hov.png"))
                        .frame(!matches!(self.active_page, ActivePage::DeviceManager))
                        .ui(ui);

                if nav_dev_man.clicked() {
                    self.active_page = ActivePage::DeviceManager;
                }

                match self.devs_setup {
                    true => {
                        if egui::ImageButton::new(egui::include_image!(
                            "../res/mcs_mainwin_lite_hov.png"
                        ))
                        .frame(!matches!(self.active_page, ActivePage::MainWindow))
                        .ui(ui)
                        .clicked()
                        {
                            self.active_page = ActivePage::MainWindow;
                        }

                        if egui::ImageButton::new(egui::include_image!("../res/mcs_devcon_lite_hov.png"))
                            .frame(!matches!(self.active_page, ActivePage::MachineConfig))
                            .ui(ui)
                            .clicked()
                        {
                            self.active_page = ActivePage::MachineConfig;
                        }
                    }
                    false => {
                        if egui::ImageButton::new(egui::include_image!(
                            "../res/mcs_mainwin_lite_disabled.png"
                        ))
                        .frame(!matches!(self.active_page, ActivePage::MainWindow))
                        .ui(ui)
                        .clicked()
                        {
                            self.dialog(DialogType::Warn, "Devices have not been setup!");
                            println!("Devices are not setup.")
                        }
                        
                        if egui::ImageButton::new(egui::include_image!(
                            "../res/mcs_devcon_lite_disabled.png"
                        ))
                        .frame(!matches!(self.active_page, ActivePage::MainWindow))
                        .ui(ui)
                        .clicked()
                        {
                            self.dialog(DialogType::Warn, "Devices have not been setup!");
                            println!("Devices are not setup.")
                        }
                    }
                }
            });

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.tabs.modal_active);

            match self.active_page {
                ActivePage::DeviceManager => {
                    ui.heading("Device Manager");

                    ///////////
                    
                    ui.add_space(15.0);
                    
                    ui.label("From the Device Manager you can select the amount and type of devices you want to connect.");
                    ui.label("Ports may display limited information about the device connected to them.");
                    
                    ui.add_space(15.0);

                    ///////////

                    if ui.button("Search for Devices").clicked() {
                        self.search_ports = true;
                    }

                    ui.label("Motion Controllers");
                    // This spinbox will determine how many motion controller entries we need.
                    // ui.add(egui::DragValue::new(&mut self.tabs.num_mc_devs)
                    //     .max_decimals(0)
                    //     .clamp_range(1..=10)
                    //     .speed(1.0));
                    ui.add(egui::Slider::new(&mut self.tabs.num_mc_devs, 1..=10));
                    
                    for i in 0..self.tabs.num_mc_devs {
                        if self.tabs.sel_mc_port.len() < i + 1 {
                            self.tabs.sel_mc_port.push("None".to_owned());
                        }

                        if self.tabs.sel_mc_model.len() < i + 1 {
                            self.tabs.sel_mc_model.push("None".to_owned());
                        }

                        if self.tabs.sel_mc_nick.len() < i + 1 {
                            self.tabs.sel_mc_nick.push("None".to_owned());
                        }

                        ui.horizontal(|ui| {
                            ui.label("Port");
                            egui::ComboBox::from_id_source(format!(
                                "Motion Controller Port {}",
                                i + 1
                            ))
                            // egui::ComboBox::new(format!("Motion Controller Port {}", i + 1), "Port")
                            .selected_text(format!("{:?}", self.tabs.sel_mc_port[i]))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                
                                // Generate combo-box items in a loop.
                                if let Ok(ports) = self.ports.as_ref() {
                                    for device in ports.iter() {
                                        ui.selectable_value(
                                            &mut self.tabs.sel_mc_port[i],
                                            device.port_name.clone(),
                                            format!(
                                                "{} {:?} {:?}",
                                                device.port_name,
                                                device.port_type,
                                                device.type_id()
                                            ),
                                        );
                                    }
                                }
                            });
                            
                            ui.label("Model");
                            egui::ComboBox::from_id_source(
                                format!("Motion Controller Model {}", i + 1),
                            )
                            .selected_text(format!("{:?}", self.tabs.sel_mc_model[i]))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                
                                // Generate combo-box items in a loop.
                                for model in self.mtn_ctrl_models.iter() {
                                    ui.selectable_value(
                                        &mut self.tabs.sel_mc_model[i],
                                        model.to_string(),
                                        model,
                                    );
                                }
                            });

                            ui.label("Nickname");
                            ui.text_edit_singleline(&mut self.tabs.sel_mc_nick[i]);
                        });
                    }

                    ui.add_space(15.0);

                    ui.label("Detectors");
                    ui.add(egui::Slider::new(&mut self.tabs.num_det_devs, 1..=2));
                    
                    for i in 0..self.tabs.num_det_devs {
                        if self.tabs.sel_det_port.len() < i + 1 {
                            self.tabs.sel_det_port.push("None".to_owned());
                        }

                        if self.tabs.sel_det_model.len() < i + 1 {
                            self.tabs.sel_det_model.push("None".to_owned());
                        }

                        if self.tabs.sel_det_nick.len() < i + 1 {
                            self.tabs.sel_det_nick.push("None".to_owned());
                        }

                        ui.horizontal(|ui| {
                            ui.label("Port");
                            egui::ComboBox::from_id_source(format!("Detector Port {}", i + 1))
                                .selected_text(format!("{:?}", self.tabs.sel_det_port[i]))
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    
                                    // Generate combo-box items in a loop.
                                    if let Ok(ports) = self.ports.as_ref() {
                                        for device in ports.iter() {
                                            ui.selectable_value(
                                                &mut self.tabs.sel_det_port[i],
                                                device.port_name.clone(),
                                                format!(
                                                    "{} {:?} {:?}",
                                                    device.port_name,
                                                    device.port_type,
                                                    device.type_id()
                                                ),
                                            );
                                        }
                                    }
                                });

                            ui.label("Model");
                            egui::ComboBox::from_id_source(format!("Detector Model {}", i + 1))
                                .selected_text(format!("{:?}", self.tabs.sel_det_model[i]))
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);

                                    // Generate combo-box items in a loop.
                                    for model in self.det_models.iter() {
                                        ui.selectable_value(
                                            &mut self.tabs.sel_det_model[i],
                                            model.to_string(),
                                            model,
                                        );
                                    }
                                });

                                ui.label("Nickname");
                                ui.text_edit_singleline(&mut self.tabs.sel_det_nick[i]);
                        });
                    }

                    ///////////

                    ui.add_space(15.0);

                    if ui
                        .button("Connect Devices")
                        .on_hover_text("Search for and connect devices as selected above.")
                        .clicked()
                    {
                        // TODO: Obviously this closure is mostly test code- requires extensive overhaul!!!
                        self.devices_loading = true;
                        self.devices_loading_progress = 0.0;

                        // Set up the devices vectors.
                        for i in 0..self.tabs.num_mc_devs {
                            let mc = MotionController::new(Box::new(drivers::mp_789a_4::Mp789a4Virtual::new(self.tabs.sel_mc_port[i].clone()).unwrap()));
                            self.connd_mtn_ctrlrs.push(mc);
                        }

                        for i in 0..self.tabs.num_det_devs {
                            let det = Detector::new(Box::new(drivers::ki_6485::Ki6485::new(self.tabs.sel_det_port[i].clone(), 10).unwrap()));
                            self.tabs.connd_detectors.push(det);

                            // Make a new vec for each detector.
                            self.tabs.detector_data.push(Vec::new());
                        }
                    }

                    if self.devices_loading {
                        ui.add(
                            egui::ProgressBar::new(self.devices_loading_progress)
                                .animate(self.devices_loading)
                                .show_percentage(),
                        );
                        self.devices_loading_progress += 0.01 / 60.0;

                        if self.devices_loading_progress >= 1.0 {
                            self.devices_loading = false;
                        }
                    }

                    // Since the GUI is immediate mode, we need to lock the ports search behind a boolean to prevent overrequesting.
                    if self.search_ports {
                        self.search_ports = false;

                        self.ports = match serialport::available_ports() {
                            Ok(ports) => Ok(ports),
                            Err(e) => {
                                self.dialog(DialogType::Error, &format!("Error: {}", e));
                                Err(e)
                            }
                        };
                    }

                    match &self.ports {
                        Ok(ports) => {
                            for p in ports {
                                ui.label(p.port_name.clone());
                            }
                        }
                        Err(e) => {
                            ui.label(format!("No ports found: {}", e));
                        }
                    }

                    ui.checkbox(&mut self.devs_setup, "Are devices setup?");
                }

                ActivePage::MainWindow => {
                    DockArea::new(&mut self.tree)
                        // .style(Style::from_egui(ctx.style().as_ref()))
                        .show(ctx, &mut self.tabs);
                }

                ActivePage::MachineConfig => {
                    ui.label("Machine Configuration");
                }
            }
        });
    }
}
