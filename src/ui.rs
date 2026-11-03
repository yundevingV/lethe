use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use eframe::egui;

use crate::bytes::format_bytes;
use crate::engine::{self, confirm_text};
use crate::model::{CacheGroup, CacheRow};
use crate::update::{self, UpdateOffer};

const BG_VOID: egui::Color32 = egui::Color32::from_rgb(0x09, 0x0A, 0x0F);
fn bg_glass() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(22, 27, 34, 160)
}

fn border_glass() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 31)
}
const AURORA_CYAN: egui::Color32 = egui::Color32::from_rgb(0x00, 0xF2, 0xFE);
const AURORA_PURPLE: egui::Color32 = egui::Color32::from_rgb(0x9D, 0x50, 0xBB);
const AURORA_PINK: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x77, 0xA9);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xF0, 0xF4, 0xF8);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x99, 0xAD);
fn dust() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(160, 175, 200, 102)
}

enum Msg {
    Scanned(Vec<CacheRow>),
    CleanStatus {
        id: String,
        message: String,
        error: bool,
        clear_selection: bool,
    },
    CleanFinished {
        failures: usize,
    },
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
    shock_at: Option<f64>,
    cleaned_bytes: u64,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl LetheApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_font(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            rows: engine::default_rows(),
            scanning: false,
            cleaning: false,
            banner: None,
            confirm: false,
            update: None,
            update_open: false,
            shock_at: None,
            cleaned_bytes: 0,
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
        self.cleaned_bytes = selected_bytes(self);
        self.cleaning = true;
        self.banner = None;
        self.confirm = false;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let mut failures = 0;
            for id in targets {
                let _ = tx.send(Msg::CleanStatus {
                    id: id.clone(),
                    message: "Lethe의 물길에 시스템 기록을 수놓는 중...".into(),
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
                Msg::CleanStatus {
                    id,
                    message,
                    error,
                    clear_selection,
                } => self.mark(&id, message, error, clear_selection),
                Msg::CleanFinished { failures } => {
                    self.cleaning = false;
                    if failures == 0 {
                        let size = format_bytes(self.cleaned_bytes);
                        self.banner = Some((
                            format!("망각의 강으로 찌꺼기를 보냈습니다. {size} 정화 완료."),
                            false,
                        ));
                        self.shock_at = Some(ctx.input(|input| input.time));
                    } else {
                        self.banner = Some((
                            "일부 기억 파편이 응결되어 정화되지 못했습니다.".into(),
                            true,
                        ));
                    }
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
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn mark(&mut self, id: &str, message: String, error: bool, clear_selection: bool) {
        let Some(row) = self.rows.iter_mut().find(|row| row.id == id) else {
            return;
        };
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
        header(ui);
        footer(self, ui);
        body(self, ui);
        confirm(self, &ctx);
        update_modal(self, &ctx);
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([780.0, 920.0])
            .with_min_inner_size([640.0, 680.0])
            .with_decorations(false)
            .with_title("Lethe"),
        ..Default::default()
    };
    eframe::run_native(
        "Lethe",
        options,
        Box::new(|cc| Ok(Box::new(LetheApp::new(cc)))),
    )
}

fn install_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/AppleSDGothicNeo.ttc") {
        fonts.font_data.insert(
            "sd-medium".into(),
            std::sync::Arc::new(face(bytes.clone(), 2)),
        );
        fonts
            .font_data
            .insert("sd-semibold".into(), std::sync::Arc::new(face(bytes, 4)));
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(list) = fonts.families.get_mut(&family) {
                list.insert(0, "sd-medium".into());
            }
        }
        fonts.families.insert(
            egui::FontFamily::Name("semibold".into()),
            vec!["sd-semibold".into(), "sd-medium".into()],
        );
    }
    let mut hero = vec!["sd-semibold".into()];
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNSMono.ttf") {
        fonts.font_data.insert(
            "sf-mono".into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        hero.insert(0, "sf-mono".into());
    }
    fonts
        .families
        .insert(egui::FontFamily::Name("hero-num".into()), hero);
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.5, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(18.0, semibold_family()),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(15.0, egui::FontFamily::Monospace),
        );
    });
}

fn face(bytes: Vec<u8>, index: u32) -> egui::FontData {
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    data.tweak.hinting = Some(true);
    data
}

fn apply_theme(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.override_text_color = Some(TEXT_PRIMARY);
        visuals.panel_fill = BG_VOID;
        visuals.window_fill = egui::Color32::from_rgba_unmultiplied(14, 18, 28, 235);
        visuals.extreme_bg_color = BG_VOID;
        visuals.faint_bg_color = bg_glass();
        visuals.code_bg_color = bg_glass();
        visuals.hyperlink_color = AURORA_CYAN;
        visuals.window_stroke = egui::Stroke::new(1.0, border_glass());
        visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 242, 254, 36);
        visuals.selection.stroke = egui::Stroke::new(1.0, AURORA_CYAN);
        visuals.widgets.noninteractive.fg_stroke.color = TEXT_SECONDARY;
        visuals.widgets.inactive.fg_stroke.color = TEXT_PRIMARY;
        visuals.widgets.hovered.fg_stroke.color = TEXT_PRIMARY;
        visuals.widgets.active.fg_stroke.color = AURORA_CYAN;
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 242, 254, 18);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 242, 254, 36);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(157, 80, 187, 48);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 242, 254, 70));
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, AURORA_CYAN);
        visuals.window_corner_radius = 14.into();
        visuals.menu_corner_radius = 12.into();
    });
}

fn semibold_family() -> egui::FontFamily {
    egui::FontFamily::Name("semibold".into())
}

fn hero_family() -> egui::FontFamily {
    egui::FontFamily::Name("hero-num".into())
}

fn semibold(text: &str, size: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .family(semibold_family())
        .color(TEXT_PRIMARY)
}

fn micro(text: &str) -> egui::RichText {
    egui::RichText::new(text).size(13.0).color(TEXT_SECONDARY)
}

fn void_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(BG_VOID)
        .inner_margin(egui::Margin::symmetric(16, 8))
}

fn header(ui: &mut egui::Ui) {
    egui::Panel::top("header")
        .resizable(false)
        .show_separator_line(false)
        .exact_size(44.0)
        .frame(void_frame())
        .show(ui, |ui| {
            let rect = ui.available_rect_before_wrap();
            let drag = ui.interact(
                rect,
                ui.scope_id().with("drag"),
                egui::Sense::click_and_drag(),
            );
            if drag.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
            ui.horizontal(|ui| {
                paint_wordmark(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if chrome_button(
                        ui,
                        "×",
                        egui::Color32::from_rgba_unmultiplied(255, 119, 169, 70),
                    )
                    .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if chrome_button(
                        ui,
                        "□",
                        egui::Color32::from_rgba_unmultiplied(0, 242, 254, 28),
                    )
                    .clicked()
                    {
                        let maximized =
                            ui.input(|input| input.viewport().maximized.unwrap_or(false));
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if chrome_button(
                        ui,
                        "–",
                        egui::Color32::from_rgba_unmultiplied(0, 242, 254, 28),
                    )
                    .clicked()
                    {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });
}

fn paint_wordmark(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(88.0, 28.0), egui::Sense::hover());
    let glow = rect.left_center() + egui::vec2(16.0, 0.0);
    ui.painter().circle_filled(
        glow,
        16.0,
        egui::Color32::from_rgba_unmultiplied(0, 242, 254, 42),
    );
    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        "Lethe",
        egui::FontId::new(22.0, semibold_family()),
        AURORA_CYAN,
    );
}

fn chrome_button(ui: &mut egui::Ui, glyph: &str, hover: egui::Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(36.0, 28.0), egui::Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, hover);
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
        TEXT_PRIMARY,
    );
    response
}

fn footer(app: &mut LetheApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("footer")
        .resizable(false)
        .show_separator_line(false)
        .frame(void_frame())
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let (label, color) = status_of(app);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(10.0, 18.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, color);
                ui.label(semibold(label, 13.0).color(color));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let text = if app.scanning {
                        "측정 중…"
                    } else {
                        "다시 측정"
                    };
                    if ui
                        .add_enabled(!app.busy(), egui::Button::new(text))
                        .clicked()
                    {
                        app.start_scan();
                    }
                });
            });
            if app.rows.iter().any(|row| row.id == "pnpm" && row.selected) {
                ui.label(micro(
                    "pnpm 용량은 스토어 전체라, 정화해도 표시된 만큼 줄지 않을 수 있습니다.",
                ));
            }
            ui.label(micro(
                "nvm, Docker 설정, 프로젝트 node_modules는 건드리지 않습니다.",
            ));
            ui.add_space(4.0);
        });
}

fn status_of(app: &LetheApp) -> (&'static str, egui::Color32) {
    if app.cleaning {
        ("정화 중", AURORA_PINK)
    } else if app.scanning {
        ("성운을 살피는 중", AURORA_CYAN)
    } else if app.banner.as_ref().is_some_and(|(_, error)| *error) {
        ("정화되지 않은 파편", AURORA_PINK)
    } else if debris_bytes(app) == 0 {
        ("System Clear", AURORA_CYAN)
    } else {
        ("잔해 감지", AURORA_PURPLE)
    }
}

fn body(app: &mut LetheApp, ui: &mut egui::Ui) {
    egui::CentralPanel::default()
        .frame(void_frame())
        .show(ui, |ui| {
            hero(app, ui);
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    select_all_bar(app, ui);
                    ui.add_space(8.0);
                    card_grid(app, ui);
                    ui.add_space(12.0);
                });
        });
}

fn hero(app: &mut LetheApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 196.0), egui::Sense::hover());
        paint_core(ui, app, rect);
        ui.add_space(4.0);
        ui.label(micro(&hero_copy(app)));
        if let Some((text, error)) = &app.banner {
            let color = if *error { AURORA_PINK } else { AURORA_CYAN };
            ui.label(egui::RichText::new(text).size(13.0).color(color));
        }
        ui.add_space(10.0);
        let count = app
            .rows
            .iter()
            .filter(|row| row.selected && row.can_select(false))
            .count();
        let enabled = count > 0 && !app.busy() && !app.confirm;
        if purge_button(ui, enabled, app.cleaning).clicked() && enabled {
            app.confirm = true;
        }
        if count == 0 && !app.busy() {
            ui.add_space(6.0);
            ui.label(micro("정화할 잔해를 카드에서 고르세요."));
        }
    });
}

fn paint_core(ui: &mut egui::Ui, app: &LetheApp, rect: egui::Rect) {
    let painter = ui.painter();
    let center = rect.center();
    let t = ui.input(|input| input.time) as f32;
    let cleaning = app.cleaning;
    let speed = if cleaning { 0.6 } else { 0.2 };
    painter.circle_filled(
        center,
        86.0,
        egui::Color32::from_rgba_unmultiplied(0, 242, 254, 16),
    );
    painter.circle_filled(
        center,
        58.0,
        egui::Color32::from_rgba_unmultiplied(157, 80, 187, 28),
    );
    painter.circle_stroke(
        center,
        72.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(0, 242, 254, 110)),
    );
    painter.circle_stroke(
        center,
        84.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(157, 80, 187, 90)),
    );
    if cleaning {
        let pulse = (t * 4.0).sin() * 0.5 + 0.5;
        painter.circle_filled(
            center,
            36.0 + pulse * 22.0,
            egui::Color32::from_rgba_unmultiplied(255, 119, 169, 48),
        );
    }
    for index in 0..3 {
        let angle = t * speed + index as f32 * std::f32::consts::TAU / 3.0;
        let point = center + egui::vec2(angle.cos(), angle.sin()) * 72.0;
        let color = match index {
            0 => AURORA_CYAN,
            1 => AURORA_PURPLE,
            _ if cleaning => AURORA_PINK,
            _ => AURORA_CYAN,
        };
        painter.circle_filled(point, 6.5, color);
    }
    let dust_count = (debris_bytes(app) / (40 * 1024 * 1024)).clamp(0, 28) as i32;
    let dissolve = if cleaning {
        (t * 0.7).sin().abs() * 34.0
    } else {
        0.0
    };
    for index in 0..dust_count {
        let angle = -t * speed * 0.45 + index as f32 * 0.57;
        let radius = 92.0 + (index % 4) as f32 * 4.0 + dissolve;
        let point = center + egui::vec2(angle.cos(), angle.sin()) * radius;
        painter.circle_filled(point, 1.8, dust());
    }
    if let Some(start) = app.shock_at {
        let age = t - start as f32;
        if (0.0..1.0).contains(&age) {
            let alpha = ((1.0 - age) * 170.0) as u8;
            painter.circle_stroke(
                center,
                24.0 + age * 260.0,
                egui::Stroke::new(
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(0, 242, 254, alpha),
                ),
            );
        }
    }
    let number = hero_number(app);
    let font = egui::FontId::new(56.0, hero_family());
    for offset in [
        egui::vec2(-1.0, 0.0),
        egui::vec2(1.0, 0.0),
        egui::vec2(0.0, -1.0),
        egui::vec2(0.0, 1.0),
    ] {
        painter.text(
            center + offset,
            egui::Align2::CENTER_CENTER,
            &number,
            font.clone(),
            egui::Color32::from_rgba_unmultiplied(0, 242, 254, 70),
        );
    }
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        number,
        font,
        TEXT_PRIMARY,
    );
}

fn hero_number(app: &LetheApp) -> String {
    let total = debris_bytes(app);
    if app.scanning && total == 0 {
        "—".into()
    } else if total == 0 {
        "0.00 MB".into()
    } else {
        format_bytes(total)
    }
}

fn hero_copy(app: &LetheApp) -> String {
    if app.cleaning {
        return "Lethe의 물길에 시스템 기록을 수놓는 중...".into();
    }
    let total = debris_bytes(app);
    if app.scanning && total == 0 {
        return "성운 잔해를 살피는 중...".into();
    }
    if total == 0 {
        return "코어가 맑습니다. 정화할 잔해가 없습니다.".into();
    }
    format!(
        "{}의 성운 잔해(Cache)가 스토리지에 얽혀있습니다.",
        format_bytes(total)
    )
}

fn debris_bytes(app: &LetheApp) -> u64 {
    app.rows
        .iter()
        .filter(|row| row.group != CacheGroup::Disk)
        .map(|row| row.bytes.unwrap_or(0))
        .sum()
}

fn selected_bytes(app: &LetheApp) -> u64 {
    app.rows
        .iter()
        .filter(|row| row.selected && row.cleanable && row.blocked_reason.is_none())
        .map(|row| row.bytes.unwrap_or(0))
        .sum()
}

fn purge_button(ui: &mut egui::Ui, enabled: bool, cleaning: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(280.0, 42.0), egui::Sense::click());
    let hot = enabled && response.hovered();
    if hot {
        ui.painter().rect_filled(
            rect.expand(7.0),
            22.0,
            egui::Color32::from_rgba_unmultiplied(0, 242, 254, 40),
        );
    }
    let fill_alpha = if enabled { 20 } else { 10 };
    ui.painter().rect(
        rect,
        21.0,
        egui::Color32::from_rgba_unmultiplied(0, 242, 254, fill_alpha),
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(0, 242, 254, if hot { 180 } else { 76 }),
        ),
        egui::StrokeKind::Inside,
    );
    let label = if cleaning {
        "정화 중…"
    } else if hot {
        "Release Into Oblivion"
    } else {
        "Purge Memory"
    };
    let color = if enabled {
        TEXT_PRIMARY
    } else {
        TEXT_SECONDARY
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::new(14.5, egui::FontFamily::Proportional),
        color,
    );
    response
}

fn select_all_bar(app: &mut LetheApp, ui: &mut egui::Ui) {
    let eligible = app.rows.iter().filter(|row| row.can_select(false)).count();
    let all_on = eligible > 0
        && app
            .rows
            .iter()
            .filter(|row| row.can_select(false))
            .all(|row| row.selected);
    let mut checked = all_on;
    let enabled = !app.busy() && eligible > 0;
    let changed = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let response = colored_checkbox(ui, &mut checked, enabled);
            ui.label(micro("전체 선택"));
            response.changed()
        })
        .inner;
    if changed {
        for row in &mut app.rows {
            if row.can_select(false) {
                row.selected = checked;
            }
        }
    }
}

fn card_grid(app: &mut LetheApp, ui: &mut egui::Ui) {
    let groups = present_groups(app);
    let mut index = 0;
    while index < groups.len() {
        let end = (index + 2).min(groups.len());
        let pair = groups[index..end].to_vec();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let span = ui.available_width();
            let width = if pair.len() == 1 {
                span
            } else {
                (span - 10.0) / 2.0
            };
            for (group, title) in pair {
                ui.vertical(|ui| {
                    ui.set_min_width(width);
                    ui.set_max_width(width);
                    glass_card(app, ui, group, title);
                });
            }
        });
        ui.add_space(10.0);
        index = end;
    }
}

fn present_groups(app: &LetheApp) -> Vec<(CacheGroup, &'static str)> {
    [
        (CacheGroup::Install, "설치 캐시"),
        (CacheGroup::Run, "실행 캐시"),
        (CacheGroup::Docker, "Docker"),
        (CacheGroup::Disk, "가상 디스크"),
    ]
    .into_iter()
    .filter(|(group, _)| !visible_rows(app, *group).is_empty())
    .collect()
}

fn visible_rows(app: &LetheApp, group: CacheGroup) -> Vec<CacheRow> {
    let mut rows: Vec<_> = app
        .rows
        .iter()
        .filter(|row| row.group == group)
        .cloned()
        .collect();
    rows.retain(|row| match row.bytes {
        Some(0) => false,
        None => !app.scanning,
        Some(_) => true,
    });
    if group == CacheGroup::Install || group == CacheGroup::Run {
        rows.sort_by(|a, b| b.bytes.unwrap_or(0).cmp(&a.bytes.unwrap_or(0)));
    }
    rows
}

fn glass_card(app: &mut LetheApp, ui: &mut egui::Ui, group: CacheGroup, title: &str) {
    let rows = visible_rows(app, group);
    if rows.is_empty() {
        return;
    }
    let sum: u64 = rows.iter().map(|row| row.bytes.unwrap_or(0)).sum();
    let section_toggle = egui::Frame::new()
        .fill(bg_glass())
        .stroke(egui::Stroke::new(1.0, border_glass()))
        .corner_radius(16)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            let toggle = card_heading(app, ui, title, &rows, sum);
            ui.add_space(8.0);
            let mut toggles = Vec::new();
            for row in &rows {
                if let Some(next) = render_row(app, ui, row) {
                    toggles.push(next);
                }
            }
            (toggle, toggles)
        })
        .inner;
    apply_toggles(app, group, section_toggle);
}

fn card_heading(
    app: &LetheApp,
    ui: &mut egui::Ui,
    title: &str,
    rows: &[CacheRow],
    sum: u64,
) -> Option<bool> {
    let selectable: Vec<_> = rows.iter().filter(|row| row.can_select(false)).collect();
    let all_on = !selectable.is_empty() && selectable.iter().all(|row| row.selected);
    let mut checked = all_on;
    let enabled = !app.busy() && !selectable.is_empty();
    let mut changed = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if rows.iter().any(|row| row.cleanable)
            && colored_checkbox(ui, &mut checked, enabled).changed()
        {
            changed = Some(checked);
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(semibold(title, 18.0));
            let size = if sum == 0 {
                "0.00 MB".into()
            } else {
                format_bytes(sum)
            };
            ui.label(micro(&format!("{size} · {}항목", rows.len())));
        });
    });
    changed
}

fn apply_toggles(
    app: &mut LetheApp,
    group: CacheGroup,
    section_toggle: (Option<bool>, Vec<(String, bool)>),
) {
    let (heading, toggles) = section_toggle;
    let busy = app.busy();
    if let Some(on) = heading {
        for row in app
            .rows
            .iter_mut()
            .filter(|row| row.group == group && row.can_select(false))
        {
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
        egui::Color32::from_rgba_unmultiplied(0, 242, 254, 28)
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(4, 3))
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
                    ui.set_max_width((ui.available_width() - 88.0).max(80.0));
                    ui.label(semibold(&row.title, 14.0));
                    ui.add(egui::Label::new(micro(&row.detail)).wrap());
                    if let Some(reason) = &row.blocked_reason {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(reason).size(13.0).color(AURORA_PINK),
                            )
                            .wrap(),
                        );
                    }
                    if let Some(status) = &row.status {
                        let color = if row.status_is_error {
                            AURORA_PINK
                        } else {
                            TEXT_SECONDARY
                        };
                        ui.label(egui::RichText::new(status).size(13.0).color(color));
                    }
                });
                let height = text.response.rect.height().max(18.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), height),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(semibold(&size_text(app, row), 14.0).color(AURORA_CYAN));
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
    let (fill, stroke) = if *checked {
        (AURORA_CYAN, AURORA_CYAN)
    } else if enabled {
        (egui::Color32::TRANSPARENT, TEXT_SECONDARY)
    } else {
        (
            egui::Color32::TRANSPARENT,
            egui::Color32::from_rgb(70, 80, 96),
        )
    };
    ui.painter().rect(
        rect,
        4.0,
        fill,
        egui::Stroke::new(1.5, stroke),
        egui::StrokeKind::Inside,
    );
    if *checked {
        let center = rect.center();
        let mark = egui::Stroke::new(2.0, BG_VOID);
        ui.painter().line_segment(
            [
                center + egui::vec2(-4.5, 0.4),
                center + egui::vec2(-1.2, 3.8),
            ],
            mark,
        );
        ui.painter().line_segment(
            [
                center + egui::vec2(-1.2, 3.8),
                center + egui::vec2(5.0, -3.6),
            ],
            mark,
        );
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

fn confirm(app: &mut LetheApp, ctx: &egui::Context) {
    if !app.confirm {
        return;
    }
    let mut open = true;
    let mut accepted = false;
    let mut cancelled = false;
    egui::Window::new("망각의 강으로 보낼까요?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(micro(
                "고른 잔해를 강에 흘려보냅니다. 다음 설치나 빌드 때 다시 받습니다.",
            ));
            ui.add_space(8.0);
            ui.label(confirm_text(&app.rows));
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Release Into Oblivion").clicked() {
                    accepted = true;
                }
                if ui.button("남긴다").clicked() {
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
    let Some(offer) = app.update.as_ref() else {
        return;
    };
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
            ui.label(micro(
                "GitHub Releases에 새 Lethe가 있습니다. 자동 설치는 없고, 받아서 교체해야 합니다.",
            ));
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
