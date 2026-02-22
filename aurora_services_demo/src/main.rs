use aurora_services::{
    NotificationBuilder, NotificationService, Orientation, SettingsService, ThemeSettings,
};
use eframe::egui::{self, Color32, RichText, ThemePreference};

fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default().with_transparent(true);
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Aurora Services Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(ThemePreference::Dark);

            let pixel_ratio = SettingsService::new()
                .and_then(|s| s.get_pixel_ratio())
                .unwrap_or(1.0) as f32;
            cc.egui_ctx.set_pixels_per_point(pixel_ratio);

            let statusbar_height = SettingsService::new()
                .and_then(|s| s.get_statusbar_height())
                .unwrap_or(41) as f32;

            Ok(Box::new(MyApp::new(statusbar_height)))
        }),
    )
}

struct MyApp {
    active_tab: Tab,
    notification: NotificationState,
    settings: SettingsState,
    status: String,
    use_system_background: bool,
    statusbar_height: f32,
}

impl MyApp {
    fn new(statusbar_height: f32) -> Self {
        Self {
            active_tab: Tab::default(),
            notification: NotificationState::default(),
            settings: SettingsState::default(),
            status: String::new(),
            use_system_background: false,
            statusbar_height,
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Tab {
    #[default]
    Notifications,
    Theme,
    Display,
    Sound,
}

#[derive(Default)]
struct NotificationState {
    app_name: String,
    summary: String,
    body: String,
    icon: String,
    timeout: i32,
    urgency: u8,
}

#[derive(Default)]
struct SettingsState {
    theme: ThemeSettings,
    orientation: Orientation,
    brightness: u32,
    sound_profile: String,
    sound_profiles: Vec<String>,
    loaded: bool,
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut top_panel = egui::TopBottomPanel::top("top_bar")
            .exact_height(self.statusbar_height)
            .show_separator_line(false);

        let mut central_panel = egui::CentralPanel::default();

        if self.use_system_background {
            let frame = egui::Frame::default().fill(Color32::TRANSPARENT);
            top_panel = top_panel.frame(frame);
            let frame = egui::Frame::default()
                .inner_margin(12.)
                .fill(Color32::TRANSPARENT);
            central_panel = central_panel.frame(frame);
        }

        top_panel.show(ctx, |_ui| {});

        central_panel.show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(10., 10.);
            ui.label(format!("Top Bar Height: {}", self.statusbar_height));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Notifications, "Notifications");
                ui.selectable_value(&mut self.active_tab, Tab::Theme, "Theme");
                ui.selectable_value(&mut self.active_tab, Tab::Display, "Display");
                ui.selectable_value(&mut self.active_tab, Tab::Sound, "Sound");
            });

            match self.active_tab {
                Tab::Notifications => self.show_notifications(ui),
                Tab::Theme => self.show_theme(ui),
                Tab::Display => self.show_display(ui),
                Tab::Sound => self.show_sound(ui),
            }

            if !self.status.is_empty() {
                ui.add_space(10.0);
                ui.label(&self.status);
            }
        });
    }
}

impl MyApp {
    fn show_notifications(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Send Notification").size(24.0));

        ui.horizontal(|ui| {
            ui.label("App name:");
            ui.text_edit_singleline(&mut self.notification.app_name);
        });

        ui.horizontal(|ui| {
            ui.label("Summary:");
            ui.text_edit_singleline(&mut self.notification.summary);
        });

        ui.horizontal(|ui| {
            ui.label("Body:");
            ui.text_edit_singleline(&mut self.notification.body);
        });

        ui.horizontal(|ui| {
            ui.label("Icon:");
            ui.text_edit_singleline(&mut self.notification.icon);
        });

        ui.horizontal(|ui| {
            ui.label("Timeout (ms):");
            ui.add(egui::DragValue::new(&mut self.notification.timeout).range(-1..=30000));
        });

        ui.horizontal(|ui| {
            ui.label("Urgency:");
            ui.add(egui::Slider::new(&mut self.notification.urgency, 0..=2));
        });

        ui.add_space(10.0);

        if ui.button("Send Notification").clicked() {
            self.send_notification();
        }

        if ui.button("Get Capabilities").clicked() {
            self.get_capabilities();
        }

        if ui.button("Get Server Info").clicked() {
            self.get_server_info();
        }
    }

    fn send_notification(&mut self) {
        match NotificationService::new() {
            Ok(service) => {
                let notification = NotificationBuilder::new()
                    .app_name(&self.notification.app_name)
                    .summary(&self.notification.summary)
                    .body(&self.notification.body)
                    .icon(&self.notification.icon)
                    .timeout(self.notification.timeout)
                    .urgency(self.notification.urgency)
                    .build();

                match service.notify(&notification) {
                    Ok(id) => {
                        self.status = format!("Notification sent with ID: {}", id);
                    }
                    Err(e) => {
                        self.status = format!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn get_capabilities(&mut self) {
        match NotificationService::new() {
            Ok(service) => match service.get_capabilities() {
                Ok(caps) => {
                    self.status = format!("Capabilities: {:?}", caps);
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn get_server_info(&mut self) {
        match NotificationService::new() {
            Ok(service) => match service.get_server_info() {
                Ok(info) => {
                    self.status = format!(
                        "Server: {} v{} by {} (spec: {})",
                        info.name, info.version, info.vendor, info.spec_version
                    );
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn show_theme(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Theme Settings").size(24.0));

        if !self.settings.loaded {
            if ui.button("Load Theme").clicked() {
                self.load_theme();
            }
        } else {
            ui.horizontal(|ui| {
                ui.label("Active Ambience:");
                if let Some(ref ambience) = self.settings.theme.active_ambience {
                    let mut s = ambience.clone();
                    ui.text_edit_singleline(&mut s);
                    self.settings.theme.active_ambience = Some(s);
                } else {
                    let mut s = String::new();
                    ui.text_edit_singleline(&mut s);
                    if !s.is_empty() {
                        self.settings.theme.active_ambience = Some(s);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Color Scheme:");
                if let Some(scheme) = self.settings.theme.color_scheme {
                    let mut s = scheme;
                    ui.add(egui::DragValue::new(&mut s));
                    self.settings.theme.color_scheme = Some(s);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Highlight Color:");
                if let Some(ref color) = self.settings.theme.highlight_color {
                    let mut s = color.clone();
                    ui.text_edit_singleline(&mut s);
                    self.settings.theme.highlight_color = Some(s);
                }
            });

            ui.add_space(10.0);

            if ui.button("Save Theme").clicked() {
                self.save_theme();
            }
        }
    }

    fn load_theme(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.get_theme() {
                Ok(theme) => {
                    self.settings.theme = theme;
                    self.settings.loaded = true;
                    self.status = "Theme loaded".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn save_theme(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.set_theme(&self.settings.theme) {
                Ok(()) => {
                    self.status = "Theme saved".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn show_display(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Display Settings").size(24.0));

        ui.horizontal(|ui| {
            ui.label("Orientation:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.settings.orientation))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.settings.orientation,
                        Orientation::Portrait,
                        "Portrait",
                    );
                    ui.selectable_value(
                        &mut self.settings.orientation,
                        Orientation::Landscape,
                        "Landscape",
                    );
                    ui.selectable_value(
                        &mut self.settings.orientation,
                        Orientation::Dynamic,
                        "Dynamic",
                    );
                });
        });

        ui.horizontal(|ui| {
            ui.label("Brightness:");
            ui.add(egui::Slider::new(&mut self.settings.brightness, 1..=100));
        });

        ui.add_space(10.0);

        if ui.button("Set Orientation").clicked() {
            self.set_orientation();
        }

        if ui.button("Set Brightness").clicked() {
            self.set_brightness();
        }

        if ui.button("Load Display Settings").clicked() {
            self.load_display();
        }
    }

    fn set_orientation(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.set_orientation_lock(self.settings.orientation) {
                Ok(()) => {
                    self.status = "Orientation set".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn set_brightness(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.set_brightness(self.settings.brightness) {
                Ok(()) => {
                    self.status = "Brightness set".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn load_display(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.get_display() {
                Ok(display) => {
                    if let Some(orient) = display.orientation_lock {
                        self.settings.orientation = orient;
                    }
                    if let Some(brightness) = display.brightness {
                        self.settings.brightness = brightness;
                    }
                    self.status = "Display settings loaded".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn show_sound(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Sound Settings").size(24.0));

        ui.horizontal(|ui| {
            ui.label("Profile:");
            ui.text_edit_singleline(&mut self.settings.sound_profile);
        });

        ui.add_space(10.0);

        if ui.button("Get Profile").clicked() {
            self.get_sound_profile();
        }

        if ui.button("Set Profile").clicked() {
            self.set_sound_profile();
        }

        if ui.button("Get Profiles").clicked() {
            self.get_sound_profiles();
        }

        if !self.settings.sound_profiles.is_empty() {
            ui.add_space(10.0);
            ui.label("Available profiles:");
            for profile in &self.settings.sound_profiles.clone() {
                if ui.button(profile).clicked() {
                    self.settings.sound_profile = profile.clone();
                }
            }
        }
    }

    fn get_sound_profile(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.get_sound_profile() {
                Ok(profile) => {
                    self.settings.sound_profile = profile;
                    self.status = "Profile loaded".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn set_sound_profile(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.set_sound_profile(&self.settings.sound_profile) {
                Ok(()) => {
                    self.status = "Profile set".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }

    fn get_sound_profiles(&mut self) {
        match SettingsService::new() {
            Ok(service) => match service.get_sound_profiles() {
                Ok(profiles) => {
                    self.settings.sound_profiles = profiles;
                    self.status = "Profiles loaded".to_string();
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            },
            Err(e) => {
                self.status = format!("Failed to create service: {}", e);
            }
        }
    }
}
