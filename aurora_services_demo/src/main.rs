use aurora_services::{NotificationBuilder, NotificationService, SettingsService};
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

            let aurora_settings = SettingsService::new().unwrap();
            if let Ok(font_settings) = aurora_settings.get_font_settings() {
                let mut style = (*cc.egui_ctx.style()).clone();
                style.text_styles.insert(
                    egui::TextStyle::Small,
                    egui::FontId::new(
                        font_settings.size_small as f32,
                        egui::FontFamily::Proportional,
                    ),
                );
                style.text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::new(
                        font_settings.size_medium as f32,
                        egui::FontFamily::Proportional,
                    ),
                );
                style.text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(
                        font_settings.size_medium as f32,
                        egui::FontFamily::Proportional,
                    ),
                );
                style.text_styles.insert(
                    egui::TextStyle::Monospace,
                    egui::FontId::new(
                        font_settings.size_medium as f32,
                        egui::FontFamily::Proportional,
                    ),
                );
                style.text_styles.insert(
                    egui::TextStyle::Heading,
                    egui::FontId::new(
                        font_settings.size_large as f32,
                        egui::FontFamily::Proportional,
                    ),
                );
                cc.egui_ctx.set_style(style);
            }

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
    status: String,
    statusbar_height: f32,
}

impl MyApp {
    fn new(statusbar_height: f32) -> Self {
        Self {
            active_tab: Tab::default(),
            notification: NotificationState::default(),
            status: format!("Status bar height: {}", statusbar_height),
            statusbar_height,
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Tab {
    #[default]
    Notifications,
    SystemInfo,
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

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let top_panel = egui::TopBottomPanel::top("top_bar")
            .exact_height(self.statusbar_height)
            .show_separator_line(false);

        let central_panel = egui::CentralPanel::default();

        top_panel.show(ctx, |_ui| {});

        central_panel.show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(10., 10.);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Notifications, "Notifications");
                ui.selectable_value(&mut self.active_tab, Tab::SystemInfo, "System Info");
            });

            ui.add_space(5.0);

            match self.active_tab {
                Tab::Notifications => self.show_notifications(ui),
                Tab::SystemInfo => self.show_system_info(ui),
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

    fn show_system_info(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("System Info").size(24.0));

        ui.add_space(10.0);

        if ui.button("Load System Info").clicked() {
            self.load_system_info();
        }

        ui.add_space(10.0);

        if let Ok(settings) = SettingsService::new() {
            if let Ok(pixel_ratio) = settings.get_pixel_ratio() {
                ui.horizontal(|ui| {
                    ui.label("Pixel Ratio:");
                    ui.label(format!("{:.2}", pixel_ratio));
                });
            }

            if let Ok(height) = settings.get_statusbar_height() {
                ui.horizontal(|ui| {
                    ui.label("Statusbar Height:");
                    ui.label(format!("{} px", height));
                });
            }

            if let Ok(fonts) = settings.get_font_settings() {
                ui.add_space(10.0);
                ui.label(RichText::new("Font Settings:").strong());
                ui.horizontal(|ui| {
                    ui.label("Family:");
                    ui.label(&fonts.family);
                });
                ui.horizontal(|ui| {
                    ui.label("Size Tiny:");
                    ui.label(format!("{} px", fonts.size_tiny));
                });
                ui.horizontal(|ui| {
                    ui.label("Size Small:");
                    ui.label(format!("{} px", fonts.size_small));
                });
                ui.horizontal(|ui| {
                    ui.label("Size Medium:");
                    ui.label(format!("{} px", fonts.size_medium));
                });
                ui.horizontal(|ui| {
                    ui.label("Size Large:");
                    ui.label(format!("{} px", fonts.size_large));
                });
                ui.horizontal(|ui| {
                    ui.label("Size Huge:");
                    ui.label(format!("{} px", fonts.size_huge));
                });
            }
        }
    }

    fn load_system_info(&mut self) {
        self.status = "System info loaded".to_string();
    }
}
