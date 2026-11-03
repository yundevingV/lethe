use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use eframe::egui;

use crate::bytes::format_bytes;
use crate::engine::{self, confirm_text};
use crate::model::{CacheGroup, CacheRow};
use crate::update::{self, UpdateOffer};

enum Msg {
    Scanned(Vec<CacheRow>),
    CleanStatus { id: String, message: String, error: bool, clear_selection: bool },
    CleanFinished { failures: usize },
    UpdateReady(Option<UpdateOffer>),
}

pub struct LetheApp {
    rows: Vec<CacheRow>,
    scanning: bool,
    cleaning: bool,
    banner: Option<(String, bool)>,
    confirm: bool,
    update: Option<UpdateOffer>,
    update_open: bool,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl LetheApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_font(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            rows: engine::default_rows(),
            scanning: false,
            cleaning: false,
            banner: None,
            confirm: false,
            update: None,
            update_open: false,
            tx,
            rx,
        };
        app.start_scan();
        app.start_update_check();
        app
    }

    fn start_update_check(&mut self) {
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let offer = update::check_for_update();
            let _ = tx.send(Msg::UpdateReady(offer));
        });
    }

    fn busy(&self) -> bool {
        self.scanning || self.cleaning
    }

    fn start_scan(&mut self) {
        if self.busy() {
            return;
        }
        self.scanning = true;
        let existing = self.rows.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Scanned(engine::scan(&existing)));
        });
    }

    fn start_clean(&mut self) {
        let targets: Vec<String> = self
            .rows
            .iter()
            .filter(|row| row.selected && row.can_select(false))
            .map(|row| row.id.clone())
            .collect();
        if targets.is_empty() || self.busy() {
            return;
        }
        self.cleaning = true;
        self.banner = None;
        self.confirm = false;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let mut failures = 0;
            for id in targets {
                let _ = tx.send(Msg::CleanStatus {
                    id: id.clone(),
                    message: "Lethe 중…".into(),
                    error: false,
                    clear_selection: false,
                });
                let result = engine::clean(&id);
                if !result.ok {
                    failures += 1;
                }
                let _ = tx.send(Msg::CleanStatus {
                    id,
                    message: result.message,
                    error: !result.ok,
                    clear_selection: result.ok,
                });
            }
            let _ = tx.send(Msg::CleanFinished { failures });
        });
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Scanned(rows) => {
                    self.rows = rows;
                    self.scanning = false;
                }
                Msg::CleanStatus { id, message, error, clear_selection } => self.mark(&id, message, error, clear_selection),
                Msg::CleanFinished { failures } => {
                    self.cleaning = false;
                    self.banner = Some(if failures == 0 {
                        ("Lethe했습니다.".into(), false)
                    } else {
                        ("Lethe하지 못했습니다.".into(), true)
                    });
                    self.start_scan();
                }
                Msg::UpdateReady(offer) => {
                    if let Some(info) = offer {
                        self.update = Some(info);
                        self.update_open = true;
                    }
                }
            }
        }
        if self.busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn mark(&mut self, id: &str, message: String, error: bool, clear_selection: bool) {
        let Some(row) = self.rows.iter_mut().find(|row| row.id == id) else { return };
        row.status = Some(message);
        row.status_is_error = error;
        if clear_selection {
            row.selected = false;
        }
    }
}

impl eframe::App for LetheApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        header(self, ui);
        footer(self, ui);
        body(self, ui);
        confirm(self, &ctx);
        update_modal(self, &ctx);
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 880.0])
            .with_min_inner_size([520.0, 560.0])
            .with_title("Lethe"),
        ..Default::default()
    };
    eframe::run_native("Lethe", options, Box::new(|cc| Ok(Box::new(LetheApp::new(cc)))))
}

fn install_font(ctx: &egui::Context) {
    let path = "/System/Library/Fonts/AppleSDGothicNeo.ttc";
    let Ok(bytes) = std::fs::read(path) else { return };
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("sd-medium".into(), std::sync::Arc::new(face(bytes.clone(), 2)));
    fonts.font_data.insert("sd-semibold".into(), std::sync::Arc::new(face(bytes, 4)));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, "sd-medium".into());
        }
    }
    fonts.families.insert(
        egui::FontFamily::Name("semibold".into()),
        vec!["sd-semibold".into(), "sd-medium".into()],
    );
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(egui::TextStyle::Small, egui::FontId::new(12.5, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(15.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(14.5, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(22.0, semibold_family()));
        style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(15.0, egui::FontFamily::Proportional));
    });
}

fn face(bytes: Vec<u8>, index: u32) -> egui::FontData {
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    data.tweak.hinting = Some(true);
    data
}

fn semibold_family() -> egui::FontFamily {
    egui::FontFamily::Name("semibold".into())
}

fn semibold(text: &str, size: f32) -> egui::RichText {
    egui::RichText::new(text).size(size).family(semibold_family())
}

fn header(app: &mut LetheApp, ui: &mut egui::Ui) {
    egui::Panel::top("header").show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(semibold("Lethe — 망각의 강", 22.0));
                ui.label(summary(app));
            });
            let label = if app.scanning { "측정 중…" } else { "다시 측정" };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.add_enabled(!app.busy(), egui::Button::new(label)).clicked() {
                    app.start_scan();
                }
            });
        });
        select_all_bar(app, ui);
        if let Some((text, error)) = &app.banner {
            ui.add_space(6.0);
            let color = if *error { egui::Color32::from_rgb(180, 40, 40) } else { egui::Color32::from_rgb(30, 120, 60) };
            ui.label(egui::RichText::new(text).color(egui::Color32::WHITE).background_color(color));
        }
        ui.add_space(6.0);
    });
}

fn summary(app: &LetheApp) -> String {
    let total: u64 = app.rows.iter().filter(|row| row.group == CacheGroup::Install || row.group == CacheGroup::Run).map(|row| row.bytes.unwrap_or(0)).sum();
    if app.scanning && total == 0 {
        "캐시 크기를 측정하는 중입니다.".into()
    } else {
        format!("캐시 합계 {}", format_bytes(total))
    }
}

fn select_all_bar(app: &mut LetheApp, ui: &mut egui::Ui) {
    let eligible = app.rows.iter().filter(|row| row.can_select(false)).count();
    let all_on = eligible > 0 && app.rows.iter().filter(|row| row.can_select(false)).all(|row| row.selected);
    let mut checked = all_on;
    let enabled = !app.busy() && eligible > 0;
    ui.add_space(4.0);
    let changed = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let response = colored_checkbox(ui, &mut checked, enabled);
        ui.label("전체 선택");
        response.changed()
    }).inner;
    if changed {
        for row in &mut app.rows {
            if row.can_select(false) {
                row.selected = checked;
            }
        }
    }
}

fn section_heading(app: &LetheApp, ui: &mut egui::Ui, title: &str, rows: &[CacheRow]) -> Option<bool> {
    let selectable: Vec<_> = rows.iter().filter(|row| row.can_select(false)).collect();
    let all_on = !selectable.is_empty() && selectable.iter().all(|row| row.selected);
    let mut checked = all_on;
    let enabled = !app.busy() && !selectable.is_empty();
    let mut changed = None;
    let width = ui.available_width();
    ui.horizontal(|ui| {
        ui.set_min_width(width);
        ui.spacing_mut().item_spacing.x = 8.0;
        if rows.iter().any(|row| row.cleanable) && colored_checkbox(ui, &mut checked, enabled).changed() {
            changed = Some(checked);
        }
        ui.label(semibold(title, 13.0).color(ui.visuals().weak_text_color()));
        let sum: u64 = rows.iter().map(|row| row.bytes.unwrap_or(0)).sum();
        if sum > 0 {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 16.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.label(egui::RichText::new(format_bytes(sum)).small().weak());
                },
            );
        }
    });
    changed
}

fn body(app: &mut LetheApp, ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            section(app, ui, CacheGroup::Disk, "가상 디스크", true);
            section(app, ui, CacheGroup::Docker, "Docker", false);
            section(app, ui, CacheGroup::Install, "설치 캐시", false);
            section(app, ui, CacheGroup::Run, "실행 캐시", false);
            ui.add_space(20.0);
        });
    });
}

fn section(app: &mut LetheApp, ui: &mut egui::Ui, group: CacheGroup, title: &str, first: bool) {
    let mut rows: Vec<_> = app.rows.iter().filter(|row| row.group == group).cloned().collect();
    rows.retain(|row| match row.bytes {
        Some(0) => false,
        None => !app.scanning,
        Some(_) => true,
    });
    if rows.is_empty() {
        return;
    }
    if group == CacheGroup::Install || group == CacheGroup::Run {
        rows.sort_by(|a, b| b.bytes.unwrap_or(0).cmp(&a.bytes.unwrap_or(0)));
    }
    ui.add_space(if first { 8.0 } else { 16.0 });
    let section_toggle = section_heading(app, ui, title, &rows);
    ui.add_space(6.0);
    let mut toggles = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if let Some(next) = render_row(app, ui, row) {
            toggles.push(next);
        }
        if index + 1 != rows.len() {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        }
    }
    let busy = app.busy();
    if let Some(on) = section_toggle {
        for row in app.rows.iter_mut().filter(|row| row.group == group && row.can_select(false)) {
            if !busy {
                row.selected = on;
            }
        }
    }
    for (id, selected) in toggles {
        if let Some(row) = app.rows.iter_mut().find(|row| row.id == id) {
            if selected && !row.can_select(busy) {
                continue;
            }
            row.selected = selected;
        }
    }
}

fn render_row(app: &LetheApp, ui: &mut egui::Ui, row: &CacheRow) -> Option<(String, bool)> {
    let mut selected = row.selected;
    let enabled = !app.busy() && (row.can_select(false) || row.selected);
    let mut toggle = None;
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(56, 132, 255, 42)
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            let row_width = ui.available_width();
            ui.horizontal(|ui| {
                ui.set_min_width(row_width);
                ui.spacing_mut().item_spacing.x = 8.0;
                if colored_checkbox(ui, &mut selected, enabled).changed() {
                    toggle = Some((row.id.clone(), selected));
                }
                let text = ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.set_max_width((ui.available_width() - 116.0).max(160.0));
                    ui.label(semibold(&row.title, 15.0));
                    ui.add(egui::Label::new(egui::RichText::new(&row.detail).small().weak()).wrap());
                    if let Some(reason) = &row.blocked_reason {
                        ui.add(egui::Label::new(egui::RichText::new(reason).small().color(egui::Color32::from_rgb(220, 70, 70))).wrap());
                    }
                    if let Some(status) = &row.status {
                        let color = if row.status_is_error {
                            egui::Color32::from_rgb(220, 70, 70)
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        ui.label(egui::RichText::new(status).small().color(color));
                    }
                });
                let height = text.response.rect.height().max(18.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), height),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(semibold(&size_text(app, row), 15.0));
                    },
                );
            });
        });
    toggle
}

fn colored_checkbox(ui: &mut egui::Ui, checked: &mut bool, enabled: bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
    if response.clicked() && enabled {
        *checked = !*checked;
        response.mark_changed();
    }
    let accent = egui::Color32::from_rgb(56, 132, 255);
    let (fill, stroke) = if *checked {
        (accent, accent)
    } else if enabled {
        (egui::Color32::TRANSPARENT, egui::Color32::from_rgb(154, 162, 174))
    } else {
        (egui::Color32::TRANSPARENT, egui::Color32::from_rgb(92, 98, 108))
    };
    ui.painter().rect(rect, 4.0, fill, egui::Stroke::new(1.5, stroke), egui::StrokeKind::Inside);
    if *checked {
        let center = rect.center();
        let mark = egui::Stroke::new(2.0, egui::Color32::WHITE);
        ui.painter().line_segment([center + egui::vec2(-4.5, 0.4), center + egui::vec2(-1.2, 3.8)], mark);
        ui.painter().line_segment([center + egui::vec2(-1.2, 3.8), center + egui::vec2(5.0, -3.6)], mark);
    }
    response
}

fn size_text(app: &LetheApp, row: &CacheRow) -> String {
    match row.bytes {
        Some(0) => "없음".into(),
        Some(bytes) => format_bytes(bytes),
        None if app.scanning => "측정 중".into(),
        None => "확인 불가".into(),
    }
}

fn footer(app: &mut LetheApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("footer").show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        ui.add_space(8.0);
        if app.rows.iter().any(|row| row.id == "pnpm" && row.selected) {
            ui.label(egui::RichText::new("pnpm 용량은 스토어 전체라, Lethe해도 표시된 만큼 줄지 않을 수 있습니다.").small().weak());
        }
        ui.horizontal(|ui| {
            let count = app.rows.iter().filter(|row| row.selected && row.can_select(false)).count();
            let total: u64 = app.rows.iter().filter(|row| row.selected && row.cleanable && row.blocked_reason.is_none()).map(|row| row.bytes.unwrap_or(0)).sum();
            let label = if app.scanning {
                "크기를 측정하는 중입니다.".into()
            } else if count == 0 {
                "Lethe할 항목을 선택하세요.".into()
            } else {
                format!("Lethe {}", format_bytes(total))
            };
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let text = if app.cleaning { "Lethe 중…" } else { "Lethe" };
                if ui.add_enabled(count > 0 && !app.busy() && !app.confirm, egui::Button::new(text)).clicked() {
                    app.confirm = true;
                }
            });
        });
        ui.label(egui::RichText::new("nvm, Docker 설정, 프로젝트 node_modules는 건드리지 않습니다. 확인 후에만 Lethe합니다.").small().weak());
        ui.add_space(6.0);
    });
}

fn confirm(app: &mut LetheApp, ctx: &egui::Context) {
    if !app.confirm {
        return;
    }
    let mut open = true;
    let mut accepted = false;
    let mut cancelled = false;
    egui::Window::new("Lethe할까요?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(confirm_text(&app.rows));
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Lethe").clicked() {
                    accepted = true;
                }
                if ui.button("취소").clicked() {
                    cancelled = true;
                }
            });
        });
    if accepted {
        app.start_clean();
    } else if cancelled || !open {
        app.confirm = false;
    }
}

fn update_modal(app: &mut LetheApp, ctx: &egui::Context) {
    let Some(offer) = app.update.as_ref() else { return };
    if !app.update_open {
        return;
    }
    let mut open = true;
    let mut skip = false;
    let mut download = false;
    let mut close = false;
    let current = update::app_version().to_string();
    let title = format!("새 버전 {} (지금 {})", offer.tag, current);
    egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("GitHub Releases에 새 Lethe가 있습니다. 자동 설치는 없고, 받아서 교체해야 합니다.")
                    .small()
                    .weak(),
            );
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .max_height(260.0)
                .show(ui, |ui| {
                    ui.add(egui::Label::new(&offer.notes).wrap());
                });
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Releases에서 받기").clicked() {
                    download = true;
                }
                if ui.button("이 버전 넘기기").clicked() {
                    skip = true;
                }
                if ui.button("닫기").clicked() {
                    close = true;
                }
            });
        });
    if download {
        update::open_release_page(&offer.page_url);
        app.update_open = false;
    } else if skip {
        let _ = update::skip_version(&offer.tag);
        app.update_open = false;
    } else if close || !open {
        app.update_open = false;
    }
}
