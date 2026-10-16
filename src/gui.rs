use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, Frame, Layout, RichText, Stroke, Vec2};

use crate::{
    battery::{self, BatteryInfo},
    cpu::{self, CpuInfo},
    disk::{self, DiskInfo},
    memory::{self, MemoryInfo},
    network::{self, NetworkInfo},
    processes::{self, ProcessInfo},
    system::{self, SystemInfo},
    temperature::{self, TemperatureInfo},
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

struct SystemSnapshot {
    cpu: CpuInfo,
    memory: MemoryInfo,
    disk: DiskInfo,
    batteries: Vec<BatteryInfo>,
    network: Vec<NetworkInfo>,
    processes: Vec<ProcessInfo>,
    system: SystemInfo,
    temperatures: Vec<TemperatureInfo>,
}

impl SystemSnapshot {
    fn collect() -> std::io::Result<Self> {
        let mut processes = processes::get_processes()?;
        processes.sort_by(|left, right| right.memory_kb.cmp(&left.memory_kb));

        Ok(Self {
            cpu: cpu::get_cpu_info()?,
            memory: memory::get_memory_info()?,
            disk: disk::get_disk_info("/")?,
            batteries: battery::get_battery_info()?,
            network: network::get_network_info()?,
            processes,
            system: system::get_system_info()?,
            temperatures: temperature::get_temperatures()?,
        })
    }
}

struct ToprsApp {
    snapshot: Option<SystemSnapshot>,
    last_refresh: Instant,
    last_updated: Option<Instant>,
    error: Option<String>,
}

impl ToprsApp {
    fn new() -> Self {
        let mut app = Self {
            snapshot: None,
            last_refresh: Instant::now() - REFRESH_INTERVAL,
            last_updated: None,
            error: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        match SystemSnapshot::collect() {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.last_updated = Some(Instant::now());
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}

impl eframe::App for ToprsApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
            self.refresh();
        }
        context.request_repaint_after(Duration::from_millis(250));

        egui::CentralPanel::default()
            .frame(Frame::new().fill(Color32::from_rgb(11, 17, 24)))
            .show(context, |ui| {
                ui.add_space(8.0);
                self.draw_header(ui);

                if let Some(error) = &self.error {
                    ui.add_space(6.0);
                    ui.colored_label(Color32::from_rgb(251, 113, 133), error);
                }

                ui.add_space(8.0);
                if let Some(snapshot) = &self.snapshot {
                    Self::draw_overview(ui, snapshot);
                    ui.add_space(8.0);
                    Self::draw_detail_cards(ui, snapshot);
                    ui.add_space(8.0);

                    let remaining_height = ui.available_height().max(100.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), remaining_height),
                        Layout::top_down(Align::Min),
                        |ui| Self::process_panel(ui, &snapshot.processes),
                    );
                } else if self.error.is_some() {
                    ui.label("Impossible de charger les informations système.");
                } else {
                    ui.spinner();
                    ui.label("Chargement des informations système…");
                }
            });
    }
}

impl ToprsApp {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("TOPRS  /  SYSTEM MONITOR")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(64, 217, 192)),
                );
                ui.heading("Centre de supervision");
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("⟳  Actualiser").clicked() {
                    self.refresh();
                }
                if let Some(updated) = self.last_updated {
                    ui.label(
                        RichText::new(format!(
                            "Mis à jour il y a {} s",
                            updated.elapsed().as_secs()
                        ))
                        .color(Color32::from_rgb(139, 155, 171)),
                    );
                }
            });
        });
    }

    fn draw_overview(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
        ui.label(
            RichText::new(format!(
                "{}  ·  {}  ·  uptime {}",
                snapshot.system.hostname,
                snapshot.system.kernel,
                format_uptime(snapshot.system.uptime_seconds)
            ))
            .color(Color32::from_rgb(139, 155, 171)),
        );
        ui.add_space(12.0);

        let available_width = ui.available_width();
        let columns = if available_width >= 1000.0 {
            4
        } else if available_width >= 600.0 {
            2
        } else {
            1
        };

        egui::Grid::new("resource_cards")
            .num_columns(columns)
            .spacing(Vec2::new(10.0, 10.0))
            .show(ui, |ui| {
                let cards = [
                    (
                        "PROCESSEUR",
                        format!("{} cœurs", snapshot.cpu.logical_cores),
                        snapshot.cpu.model.clone(),
                        None,
                        Color32::from_rgb(64, 217, 192),
                    ),
                    (
                        "MÉMOIRE",
                        format!("{:.1} %", snapshot.memory.usage_percent),
                        format!(
                            "{} utilisés · {} disponibles",
                            format_bytes(snapshot.memory.used.saturating_mul(1024)),
                            format_bytes(snapshot.memory.available.saturating_mul(1024))
                        ),
                        Some(snapshot.memory.usage_percent as f32 / 100.0),
                        Color32::from_rgb(64, 217, 192),
                    ),
                    (
                        "STOCKAGE · /",
                        format!("{:.1} %", disk_percent(&snapshot.disk)),
                        format!(
                            "{} utilisés / {} au total · {} disponibles",
                            format_bytes(snapshot.disk.used_bytes),
                            format_bytes(snapshot.disk.total_bytes),
                            format_bytes(snapshot.disk.available_bytes)
                        ),
                        Some(disk_percent(&snapshot.disk) as f32 / 100.0),
                        Color32::from_rgb(182, 156, 255),
                    ),
                    (
                        "SWAP",
                        if snapshot.memory.swap_total == 0 {
                            "Non configuré".to_string()
                        } else {
                            format!("{:.1} %", swap_percent(&snapshot.memory))
                        },
                        format!(
                            "{} / {}",
                            format_bytes(snapshot.memory.swap_used.saturating_mul(1024)),
                            format_bytes(snapshot.memory.swap_total.saturating_mul(1024))
                        ),
                        Some(swap_percent(&snapshot.memory) as f32 / 100.0),
                        Color32::from_rgb(242, 189, 98),
                    ),
                ];

                for (index, (title, value, detail, percent, color)) in cards.into_iter().enumerate()
                {
                    Self::resource_card(ui, title, &value, &detail, percent, color);
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn resource_card(
        ui: &mut egui::Ui,
        title: &str,
        value: &str,
        detail: &str,
        percent: Option<f32>,
        color: Color32,
    ) {
        Frame::group(ui.style())
            .fill(Color32::from_rgb(17, 26, 36))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 53, 68)))
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() / 4.0).max(150.0));
                ui.label(
                    RichText::new(title)
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(139, 155, 171)),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(value).size(21.0).strong().color(color));
                if let Some(percent) = percent {
                    ui.add(
                        egui::ProgressBar::new(percent.clamp(0.0, 1.0))
                            .desired_height(5.0)
                            .fill(color),
                    );
                } else {
                    ui.add_space(5.0);
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(detail)
                        .size(10.0)
                        .color(Color32::from_rgb(139, 155, 171)),
                );
            });
    }

    fn draw_detail_cards(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
        ui.columns(2, |columns| {
            Self::battery_panel(&mut columns[0], &snapshot.batteries);
            Self::system_panel(&mut columns[1], &snapshot.system);
        });
        ui.add_space(8.0);
        ui.columns(2, |columns| {
            Self::network_panel(&mut columns[0], &snapshot.network);
            Self::temperature_panel(&mut columns[1], &snapshot.temperatures);
        });
    }

    fn process_panel(ui: &mut egui::Ui, processes: &[ProcessInfo]) {
        Frame::group(ui.style())
            .fill(Color32::from_rgb(17, 26, 36))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 53, 68)))
            .show(ui, |ui| {
                ui.heading(format!("PROCESSUS  ·  {}", processes.len()));
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("gui_processes")
                    .auto_shrink([false, false])
                    .max_height(ui.available_height().max(80.0))
                    .show(ui, |ui| {
                        egui::Grid::new("process_table")
                            .striped(true)
                            .num_columns(4)
                            .spacing(Vec2::new(18.0, 4.0))
                            .show(ui, |ui| {
                                for heading in ["PID", "NOM", "ÉTAT", "MÉMOIRE"] {
                                    ui.label(
                                        RichText::new(heading)
                                            .size(10.0)
                                            .strong()
                                            .color(Color32::from_rgb(98, 168, 255)),
                                    );
                                }
                                ui.end_row();

                                for process in processes {
                                    ui.label(process.pid.to_string());
                                    ui.label(&process.name);
                                    ui.label(&process.state);
                                    ui.label(format_bytes(process.memory_kb.saturating_mul(1024)));
                                    ui.end_row();
                                }
                            });
                    });
            });
    }

    fn battery_panel(ui: &mut egui::Ui, batteries: &[BatteryInfo]) {
        Self::info_panel(ui, "ÉNERGIE", |ui| {
            if batteries.is_empty() {
                ui.label("Aucune batterie détectée");
            }
            for battery in batteries {
                ui.label(format!(
                    "{}  ·  {}  ·  {}",
                    battery.name,
                    battery
                        .capacity
                        .map_or_else(|| "—".to_string(), |value| format!("{value}%")),
                    battery.status.as_deref().unwrap_or("Inconnu")
                ));
            }
        });
    }

    fn network_panel(ui: &mut egui::Ui, interfaces: &[NetworkInfo]) {
        Self::info_panel(
            ui,
            &format!("RÉSEAU  ·  {} interfaces", interfaces.len()),
            |ui| {
                if interfaces.is_empty() {
                    ui.label("Aucune interface détectée");
                    return;
                }

                egui::Grid::new("network_interfaces")
                    .num_columns(3)
                    .striped(true)
                    .spacing(Vec2::new(20.0, 5.0))
                    .show(ui, |ui| {
                        for heading in ["INTERFACE", "REÇU", "TRANSMIS"] {
                            ui.label(
                                RichText::new(heading)
                                    .size(10.0)
                                    .strong()
                                    .color(Color32::from_rgb(98, 168, 255)),
                            );
                        }
                        ui.end_row();

                        for interface in interfaces {
                            ui.label(RichText::new(&interface.interface).strong());
                            ui.label(format_bytes(interface.received_bytes));
                            ui.label(format_bytes(interface.transmitted_bytes));
                            ui.end_row();
                        }
                    });
            },
        );
    }

    fn temperature_panel(ui: &mut egui::Ui, temperatures: &[TemperatureInfo]) {
        Self::info_panel(
            ui,
            &format!("THERMIQUE  ·  {} capteurs", temperatures.len()),
            |ui| {
                if temperatures.is_empty() {
                    ui.label("Aucun capteur détecté");
                    return;
                }

                egui::Grid::new("thermal_sensors")
                    .num_columns(2)
                    .striped(true)
                    .spacing(Vec2::new(20.0, 5.0))
                    .show(ui, |ui| {
                        for sensor in temperatures {
                            ui.label(&sensor.sensor);
                            ui.label(
                                RichText::new(format!("{:.1} °C", sensor.temperature_celsius))
                                    .strong()
                                    .color(Color32::from_rgb(242, 189, 98)),
                            );
                            ui.end_row();
                        }
                    });
            },
        );
    }

    fn system_panel(ui: &mut egui::Ui, system: &SystemInfo) {
        Self::info_panel(ui, "SYSTÈME", |ui| {
            ui.label(format!("Hôte  ·  {}", system.hostname));
            ui.label(format!("Noyau  ·  {}", system.kernel));
            ui.label(format!(
                "Uptime  ·  {}",
                format_uptime(system.uptime_seconds)
            ));
        });
    }

    fn info_panel(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
        Frame::group(ui.style())
            .fill(Color32::from_rgb(17, 26, 36))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 53, 68)))
            .show(ui, |ui| {
                ui.set_min_height(132.0);
                ui.label(
                    RichText::new(title)
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(64, 217, 192)),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt(("info_panel", title))
                    .auto_shrink([false, false])
                    .max_height(68.0)
                    .show(ui, content);
            });
    }
}

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("TOPRS — Centre de supervision")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([900.0, 650.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TOPRS",
        options,
        Box::new(|creation_context| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::from_rgb(11, 17, 24);
            visuals.window_fill = Color32::from_rgb(17, 26, 36);
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(17, 26, 36);
            visuals.selection.bg_fill = Color32::from_rgb(35, 105, 95);
            creation_context.egui_ctx.set_visuals(visuals);
            Ok(Box::new(ToprsApp::new()))
        }),
    )
}

fn disk_percent(disk: &DiskInfo) -> f64 {
    if disk.total_bytes == 0 {
        0.0
    } else {
        disk.used_bytes as f64 / disk.total_bytes as f64 * 100.0
    }
}

fn swap_percent(memory: &MemoryInfo) -> f64 {
    if memory.swap_total == 0 {
        0.0
    } else {
        memory.swap_used as f64 / memory.swap_total as f64 * 100.0
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn format_uptime(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    format!("{days}j {hours:02}h {minutes:02}m")
}
