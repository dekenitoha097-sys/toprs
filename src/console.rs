use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState, Wrap},
};

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
const MAX_NETWORK_INTERFACES: usize = 3;
const MAX_TEMPERATURES: usize = 4;

type DashboardTerminal = Terminal<CrosstermBackend<Stdout>>;

struct DashboardData {
    cpu: CpuInfo,
    memory: MemoryInfo,
    disk: DiskInfo,
    batteries: Vec<BatteryInfo>,
    network: Vec<NetworkInfo>,
    processes: Vec<ProcessInfo>,
    system: SystemInfo,
    temperatures: Vec<TemperatureInfo>,
}

impl DashboardData {
    fn load() -> io::Result<Self> {
        let mut processes = processes::get_processes()?;
        processes.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));

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

struct App {
    data: DashboardData,
    selected_process: usize,
    last_refresh: Instant,
    refresh_error: Option<String>,
}

impl App {
    fn new(data: DashboardData) -> Self {
        Self {
            data,
            selected_process: 0,
            last_refresh: Instant::now(),
            refresh_error: None,
        }
    }

    fn refresh(&mut self) {
        match DashboardData::load() {
            Ok(data) => {
                self.data = data;
                self.selected_process = self
                    .selected_process
                    .min(self.data.processes.len().saturating_sub(1));
                self.refresh_error = None;
            }
            Err(error) => self.refresh_error = Some(error.to_string()),
        }

        self.last_refresh = Instant::now();
    }

    fn move_selection(&mut self, offset: isize) {
        if self.data.processes.is_empty() {
            self.selected_process = 0;
            return;
        }

        let last = self.data.processes.len() - 1;
        self.selected_process = self
            .selected_process
            .saturating_add_signed(offset)
            .min(last);
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let data = DashboardData::load()?;
    enable_raw_mode()?;

    let mut terminal = match Terminal::new(CrosstermBackend::new(io::stdout())) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
    };

    if let Err(error) = execute!(terminal.backend_mut(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }

    let result = run_app(&mut terminal, App::new(data));
    let raw_mode_result = disable_raw_mode();
    let screen_result = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let cursor_result = terminal.show_cursor();

    result?;
    raw_mode_result?;
    screen_result?;
    cursor_result?;
    Ok(())
}

fn run_app(terminal: &mut DashboardTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
                    _ => {}
                }
            }
        }

        if app.last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh();
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < 72 || area.height < 20 {
        draw_small_terminal(frame, area);
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(frame, sections[0], app);
    draw_metrics(frame, sections[1], app);
    draw_details(frame, sections[2], app);
    draw_footer(frame, sections[3], app);
}

fn draw_small_terminal(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" TOPRS ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let message = Paragraph::new(
        "Agrandissez le terminal pour afficher le tableau de bord.\n\
         Taille minimale : 72 colonnes x 20 lignes.\n\n\
         q / Échap : quitter",
    )
    .block(block)
    .alignment(ratatui::layout::Alignment::Center)
    .wrap(Wrap { trim: true });
    frame.render_widget(message, area);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let identity = format!(
        " {}  •  {}  •  uptime {} ",
        app.data.system.hostname,
        app.data.system.kernel,
        format_uptime(app.data.system.uptime_seconds)
    );
    let header = Paragraph::new(identity)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .title(" TOPRS  /  SYSTEM MONITOR ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(header, area);
}

fn draw_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    draw_cpu_card(frame, columns[0], &app.data.cpu);
    draw_memory_card(frame, columns[1], &app.data.memory);
    draw_swap_card(frame, columns[2], &app.data.memory);
    draw_disk_card(frame, columns[3], &app.data.disk);
}

fn draw_cpu_card(frame: &mut Frame, area: Rect, cpu: &CpuInfo) {
    let text = Paragraph::new(format!(
        "{}\n{} cœurs logiques",
        cpu.model, cpu.logical_cores
    ))
    .style(Style::default().fg(Color::White))
    .wrap(Wrap { trim: true })
    .block(card_block("CPU", Color::Cyan));
    frame.render_widget(text, area);
}

fn draw_memory_card(frame: &mut Frame, area: Rect, memory: &MemoryInfo) {
    draw_gauge_card(
        frame,
        area,
        "MÉMOIRE",
        memory.usage_percent,
        &format!(
            "{} / {}",
            format_bytes(memory.used.saturating_mul(1024)),
            format_bytes(memory.total.saturating_mul(1024))
        ),
        &format!(
            "Disponible {}",
            format_bytes(memory.available.saturating_mul(1024))
        ),
        Color::Green,
    );
}

fn draw_swap_card(frame: &mut Frame, area: Rect, memory: &MemoryInfo) {
    let percent = if memory.swap_total == 0 {
        0.0
    } else {
        memory.swap_used as f64 / memory.swap_total as f64 * 100.0
    };
    let label = if memory.swap_total == 0 {
        "Aucun swap configuré".to_string()
    } else {
        format!(
            "{} / {}",
            format_bytes(memory.swap_used.saturating_mul(1024)),
            format_bytes(memory.swap_total.saturating_mul(1024))
        )
    };

    draw_gauge_card(
        frame,
        area,
        "SWAP",
        percent,
        &label,
        "Utilisation actuelle",
        Color::Yellow,
    );
}

fn draw_disk_card(frame: &mut Frame, area: Rect, disk: &DiskInfo) {
    let percent = if disk.total_bytes == 0 {
        0.0
    } else {
        disk.used_bytes as f64 / disk.total_bytes as f64 * 100.0
    };

    draw_gauge_card(
        frame,
        area,
        "DISQUE  /",
        percent,
        &format!(
            "{} / {}",
            format_bytes(disk.used_bytes),
            format_bytes(disk.total_bytes)
        ),
        &format!(
            "Libre {} · dispo {}",
            format_bytes(disk.free_bytes),
            format_bytes(disk.available_bytes)
        ),
        Color::Magenta,
    );
}

fn draw_gauge_card(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    percent: f64,
    label: &str,
    details: &str,
    color: Color,
) {
    let block = card_block(title, color);
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_area);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .percent(percent.clamp(0.0, 100.0).round() as u16)
        .label(format!("{:.0}%", percent))
        .use_unicode(true);
    frame.render_widget(gauge, rows[0]);
    frame.render_widget(
        Paragraph::new(label)
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(details)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center),
        rows[2],
    );
}

fn draw_details(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    draw_process_table(frame, columns[0], app);

    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(columns[1]);

    draw_batteries(frame, panels[0], &app.data.batteries);
    draw_network(frame, panels[1], &app.data.network);
    draw_temperatures(frame, panels[2], &app.data.temperatures);
}

fn draw_process_table(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new([
        Cell::from("PID"),
        Cell::from("PROCESSUS"),
        Cell::from("ÉTAT"),
        Cell::from("MÉMOIRE"),
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(0);

    let rows = app.data.processes.iter().map(|process| {
        Row::new([
            Cell::from(process.pid.to_string()),
            Cell::from(process.name.clone()),
            Cell::from(process.state.clone()),
            Cell::from(format_bytes(process.memory_kb.saturating_mul(1024))),
        ])
    });

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(48),
        Constraint::Length(8),
        Constraint::Percentage(32),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(card_block("PROCESSUS  /  TRI MÉMOIRE", Color::Cyan))
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(24, 48, 64))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut state = TableState::default();
    if !app.data.processes.is_empty() {
        state.select(Some(app.selected_process));
    }
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_batteries(frame: &mut Frame, area: Rect, batteries: &[BatteryInfo]) {
    let lines = if batteries.is_empty() {
        vec!["Aucune batterie détectée".to_string()]
    } else {
        batteries
            .iter()
            .map(|battery| {
                let capacity = battery
                    .capacity
                    .map_or_else(|| "—".to_string(), |value| format!("{value}%"));
                let status = battery.status.as_deref().unwrap_or("inconnu");
                format!("{}  {}  {}", battery.name, capacity, status)
            })
            .collect()
    };
    draw_info_panel(frame, area, "ÉNERGIE", Color::Yellow, lines);
}

fn draw_network(frame: &mut Frame, area: Rect, interfaces: &[NetworkInfo]) {
    let lines = if interfaces.is_empty() {
        vec!["Aucune interface détectée".to_string()]
    } else {
        interfaces
            .iter()
            .take(MAX_NETWORK_INTERFACES)
            .map(|interface| {
                format!(
                    "{}  ↓ {}  ↑ {}",
                    interface.interface,
                    format_bytes(interface.received_bytes),
                    format_bytes(interface.transmitted_bytes)
                )
            })
            .collect()
    };
    draw_info_panel(frame, area, "RÉSEAU  /  CUMULÉ", Color::Green, lines);
}

fn draw_temperatures(frame: &mut Frame, area: Rect, temperatures: &[TemperatureInfo]) {
    let lines = if temperatures.is_empty() {
        vec!["Aucun capteur détecté".to_string()]
    } else {
        temperatures
            .iter()
            .take(MAX_TEMPERATURES)
            .map(|temperature| {
                format!(
                    "{}  {:.1} °C",
                    temperature.sensor, temperature.temperature_celsius
                )
            })
            .collect()
    };
    draw_info_panel(frame, area, "THERMIQUE", Color::Red, lines);
}

fn draw_info_panel(frame: &mut Frame, area: Rect, title: &str, color: Color, lines: Vec<String>) {
    let text = Paragraph::new(lines.join("\n"))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .block(card_block(title, color));
    frame.render_widget(text, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(error) = &app.refresh_error {
        format!(" ERREUR DE RAFRAÎCHISSEMENT : {error}  •  q quitter ")
    } else {
        " ↑/↓ ou j/k naviguer  •  r actualiser  •  q quitter  •  actualisation auto 2 s "
            .to_string()
    };
    let color = if app.refresh_error.is_some() {
        Color::Red
    } else {
        Color::Gray
    };
    frame.render_widget(Paragraph::new(text).style(Style::default().fg(color)), area);
}

fn card_block(title: &str, color: Color) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
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
