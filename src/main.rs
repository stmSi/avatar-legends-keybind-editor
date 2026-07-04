#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod keys;

use config::{Binding, ButtonConfig, ConflictGroup};
use eframe::egui;
use egui::{Color32, RichText, Stroke};
use keys::{KeyScanner, KeySnapshot, key_label};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_CONFIG: &str = r"C:\Games\Steam\steamapps\common\Avatar Legends The Fighting Game  Playtest\data\button_config.ini";

const BG: Color32 = Color32::from_rgb(16, 19, 27);
const PANEL: Color32 = Color32::from_rgb(23, 27, 38);
const CARD: Color32 = Color32::from_rgb(30, 35, 48);
const CARD_HOVER: Color32 = Color32::from_rgb(39, 45, 60);
const ACCENT: Color32 = Color32::from_rgb(92, 210, 190);
const ACCENT_DIM: Color32 = Color32::from_rgb(41, 94, 91);
const TEXT_MUTED: Color32 = Color32::from_rgb(158, 166, 184);
const CONFLICT: Color32 = Color32::from_rgb(245, 168, 72);
const CONFLICT_DIM: Color32 = Color32::from_rgb(91, 58, 29);
const ACTIVE: Color32 = Color32::from_rgb(80, 214, 136);
const ACTIVE_DIM: Color32 = Color32::from_rgb(28, 90, 60);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Configure,
    Test,
}

#[derive(Clone)]
enum CaptureTarget {
    Replace { line_index: usize, label: String },
    Add { player: u8, action: String },
}

struct Status {
    message: String,
    is_error: bool,
}

struct BindStudio {
    document: Option<ButtonConfig>,
    path_input: String,
    screen: Screen,
    scanner: KeyScanner,
    capture: Option<CaptureTarget>,
    hovered_conflict: Option<String>,
    status: Option<Status>,
}

impl BindStudio {
    fn new(ctx: &egui::Context, startup_path: PathBuf) -> Self {
        configure_style(ctx);
        let mut app = Self {
            document: None,
            path_input: startup_path.display().to_string(),
            screen: Screen::Configure,
            scanner: KeyScanner::new(),
            capture: None,
            hovered_conflict: None,
            status: None,
        };
        if startup_path.exists() {
            app.load(startup_path);
            app.status = None;
        }
        app
    }

    fn load(&mut self, path: PathBuf) {
        match ButtonConfig::load(&path) {
            Ok(document) => {
                self.path_input = path.display().to_string();
                self.document = Some(document);
                self.hovered_conflict = None;
                self.status = Some(Status {
                    message: "Config loaded".to_owned(),
                    is_error: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    message: format!("Could not open config: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn save(&mut self) {
        let Some(document) = self.document.as_mut() else {
            return;
        };
        match document.save() {
            Ok(backup) => {
                self.status = Some(Status {
                    message: format!("Saved · Backup: {}", backup.display()),
                    is_error: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    message: format!("Could not save config: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn backup_now(&mut self) {
        let Some(document) = self.document.as_ref() else {
            return;
        };
        match document.create_backup() {
            Ok(path) => {
                self.status = Some(Status {
                    message: format!("Backup created: {}", path.display()),
                    is_error: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    message: format!("Could not create backup: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn restore_backup(&mut self, path: PathBuf) {
        let Some(document) = self.document.as_mut() else {
            return;
        };
        match document.restore_from(&path) {
            Ok(()) => {
                self.hovered_conflict = None;
                self.status = Some(Status {
                    message: format!(
                        "Backup loaded from {} · Review it, then Save to apply",
                        path.display()
                    ),
                    is_error: false,
                });
            }
            Err(error) => {
                self.status = Some(Status {
                    message: format!("Could not restore backup: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn apply_captured_key(&mut self, token: &str) {
        let Some(target) = self.capture.take() else {
            return;
        };
        let Some(document) = self.document.as_mut() else {
            return;
        };

        let changed = match target {
            CaptureTarget::Replace { line_index, .. } => document.set_source(line_index, token),
            CaptureTarget::Add { player, action } => {
                document.add_keyboard_binding(player, &action, token)
            }
        };
        if changed {
            self.status = None;
        }
    }

    fn handle_capture(&mut self, snapshot: &KeySnapshot) {
        if self.capture.is_none() {
            return;
        }
        if snapshot.newly_pressed.iter().any(|key| key == "KEY_ESCAPE") {
            self.capture = None;
            return;
        }
        if let Some(token) = snapshot
            .newly_pressed
            .iter()
            .find(|token| token.as_str() != "KEY_ESCAPE")
        {
            self.apply_captured_key(token);
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        if let Some(path) = dropped.into_iter().find_map(|file| file.path) {
            self.load(path);
        }
    }

    fn top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(PANEL)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("AVATAR BIND STUDIO")
                                    .size(20.0)
                                    .strong()
                                    .color(ACCENT),
                            );
                            if self.document.as_ref().is_some_and(|document| document.dirty) {
                                ui.label(RichText::new("● UNSAVED").small().color(CONFLICT));
                            }
                        });
                        ui.label(
                            RichText::new("Keyboard mapping editor · controller lines and macros stay untouched")
                                .small()
                                .color(TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let path_edit = egui::TextEdit::singleline(&mut self.path_input)
                        .hint_text("Drop or choose button_config.ini");
                    let path_width = (ui.available_width() - 310.0).max(140.0);
                    ui.add_sized([path_width, 32.0], path_edit);

                    if ui.button("Browse…").clicked() {
                        let mut picker =
                            rfd::FileDialog::new().add_filter("Button config", &["ini"]);
                        if let Some(directory) = browse_start_directory(&self.path_input) {
                            picker = picker.set_directory(directory);
                        }
                        if let Some(path) = picker.pick_file() {
                            self.load(path);
                        }
                    }
                    if ui.button("Reload").clicked() {
                        self.load(PathBuf::from(self.path_input.trim()));
                    }
                    let can_save = self.document.is_some();
                    if ui
                        .add_enabled(can_save, egui::Button::new(RichText::new("Save").strong()))
                        .clicked()
                    {
                        self.save();
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("i").strong().color(ACCENT));
                    ui.label(
                        RichText::new("Close the game before saving; it writes this file again when it exits.")
                            .small()
                            .color(TEXT_MUTED),
                    );
                    ui.separator();
                    if ui.button("Backup now").clicked() {
                        self.backup_now();
                    }
                    if ui.button("Restore backup…").clicked() {
                        let backup_dir = self
                            .document
                            .as_ref()
                            .map(ButtonConfig::backup_dir)
                            .unwrap_or_default();
                        let mut picker = rfd::FileDialog::new().add_filter("Config backup", &["ini"]);
                        if backup_dir.exists() {
                            picker = picker.set_directory(backup_dir);
                        }
                        if let Some(path) = picker.pick_file() {
                            self.restore_backup(path);
                        }
                    }
                });

                if let Some(status) = &self.status {
                    ui.add_space(6.0);
                    let color = if status.is_error {
                        Color32::from_rgb(248, 113, 113)
                    } else {
                        ACCENT
                    };
                    ui.label(RichText::new(&status.message).small().color(color));
                }
            });
    }

    fn navigation_and_conflicts(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(BG)
            .inner_margin(egui::Margin::symmetric(18, 10))
            .show(ui, |ui| {
                let conflicts = self
                    .document
                    .as_ref()
                    .map(ButtonConfig::conflict_groups)
                    .unwrap_or_default();

                if !conflicts.is_empty() {
                    let affected = conflicts
                        .iter()
                        .map(|group| group.members.len())
                        .sum::<usize>();
                    egui::Frame::new()
                        .fill(CONFLICT_DIM)
                        .stroke(Stroke::new(1.0, CONFLICT))
                        .corner_radius(8)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("!").strong().color(CONFLICT));
                                ui.label(
                                    RichText::new(format!(
                                        "{} conflicting {} · {affected} affected bindings",
                                        conflicts.len(),
                                        if conflicts.len() == 1 { "key" } else { "keys" }
                                    ))
                                    .strong()
                                    .color(CONFLICT),
                                );
                                ui.label(
                                    RichText::new(
                                        "Hover an outlined key to reveal every conflict.",
                                    )
                                    .small()
                                    .color(Color32::from_rgb(239, 205, 163)),
                                );
                            });
                        });
                    ui.add_space(9.0);
                }

                ui.horizontal(|ui| {
                    tab_button(ui, &mut self.screen, Screen::Configure, "Configure");
                    tab_button(ui, &mut self.screen, Screen::Test, "Test inputs");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Ctrl+S to save · Drop an .ini file anywhere")
                                .small()
                                .color(TEXT_MUTED),
                        );
                    });
                });
            });
    }

    fn configure_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.document.is_none() {
            empty_state(ui);
            return;
        }

        let conflicts = self.document.as_ref().unwrap().conflict_groups();
        let conflict_map = conflict_lookup(&conflicts);
        let previous_hover = self.hovered_conflict.clone();
        let mut next_hover = None;
        let content_width = ui.available_width();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(content_width);
            ui.columns(2, |columns| {
                self.player_editor(
                    &mut columns[0],
                    1,
                    &conflict_map,
                    previous_hover.as_deref(),
                    &mut next_hover,
                );
                self.player_editor(
                    &mut columns[1],
                    2,
                    &conflict_map,
                    previous_hover.as_deref(),
                    &mut next_hover,
                );
            });
        });

        if next_hover != self.hovered_conflict {
            self.hovered_conflict = next_hover;
            ctx.request_repaint();
        }
    }

    fn player_editor(
        &mut self,
        ui: &mut egui::Ui,
        player: u8,
        conflict_map: &HashMap<usize, String>,
        active_hover: Option<&str>,
        next_hover: &mut Option<String>,
    ) {
        let bindings = self
            .document
            .as_ref()
            .map(|document| document.keyboard_bindings(player))
            .unwrap_or_default();
        let actions = ordered_actions(&bindings);
        let mut remove_line = None;

        egui::Frame::new()
            .fill(PANEL)
            .stroke(Stroke::new(1.0, Color32::from_rgb(46, 53, 70)))
            .corner_radius(12)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("PLAYER {player}"))
                            .size(17.0)
                            .strong()
                            .color(if player == 1 {
                                ACCENT
                            } else {
                                Color32::from_rgb(142, 166, 255)
                            }),
                    );
                    ui.label(
                        RichText::new(format!("{} keyboard bindings", bindings.len()))
                            .small()
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(8.0);

                for (section, section_actions) in grouped_actions(&actions) {
                    ui.label(RichText::new(section).small().strong().color(TEXT_MUTED));
                    ui.add_space(4.0);
                    for action in section_actions {
                        let action_bindings = bindings
                            .iter()
                            .filter(|binding| binding.action == action)
                            .cloned()
                            .collect::<Vec<_>>();
                        let row_is_highlighted = action_bindings.iter().any(|binding| {
                            conflict_map
                                .get(&binding.line_index)
                                .is_some_and(|key| Some(key.as_str()) == active_hover)
                        });
                        let row_fill = if row_is_highlighted {
                            CONFLICT_DIM
                        } else {
                            CARD
                        };

                        egui::Frame::new()
                            .fill(row_fill)
                            .corner_radius(8)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (name, description) = action_label(&action);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(125.0, 34.0),
                                        egui::Layout::top_down(egui::Align::LEFT),
                                        |ui| {
                                            ui.label(RichText::new(name).strong());
                                            ui.label(
                                                RichText::new(description)
                                                    .small()
                                                    .color(TEXT_MUTED),
                                            );
                                        },
                                    );

                                    ui.horizontal_wrapped(|ui| {
                                        for binding in &action_bindings {
                                            let conflict_key =
                                                conflict_map.get(&binding.line_index);
                                            let is_hot = conflict_key.is_some_and(|key| {
                                                Some(key.as_str()) == active_hover
                                            });
                                            let fill = if is_hot {
                                                CONFLICT
                                            } else if conflict_key.is_some() {
                                                CONFLICT_DIM
                                            } else {
                                                ACCENT_DIM
                                            };
                                            let text_color = if is_hot {
                                                Color32::from_rgb(32, 24, 14)
                                            } else {
                                                Color32::WHITE
                                            };
                                            let stroke = if conflict_key.is_some() {
                                                Stroke::new(1.5, CONFLICT)
                                            } else {
                                                Stroke::NONE
                                            };
                                            let response = ui.add(
                                                egui::Button::new(
                                                    RichText::new(key_label(&binding.source))
                                                        .color(text_color),
                                                )
                                                .fill(fill)
                                                .stroke(stroke),
                                            );
                                            if response.clicked() {
                                                self.capture = Some(CaptureTarget::Replace {
                                                    line_index: binding.line_index,
                                                    label: format!(
                                                        "Player {player} · {}",
                                                        action_label(&binding.action).0
                                                    ),
                                                });
                                            }
                                            if response.hovered()
                                                && let Some(key) = conflict_key
                                            {
                                                *next_hover = Some(key.clone());
                                                response.on_hover_text(
                                                    "This key triggers more than one binding",
                                                );
                                            }
                                            if ui
                                                .small_button("×")
                                                .on_hover_text("Remove this binding")
                                                .clicked()
                                            {
                                                remove_line = Some(binding.line_index);
                                            }
                                        }
                                        if ui
                                            .small_button("+ Add")
                                            .on_hover_text("Add another key for this action")
                                            .clicked()
                                        {
                                            self.capture = Some(CaptureTarget::Add {
                                                player,
                                                action: action.clone(),
                                            });
                                        }
                                    });
                                });
                            });
                        ui.add_space(5.0);
                    }
                    ui.add_space(7.0);
                }
            });

        if let Some(line_index) = remove_line
            && self
                .document
                .as_mut()
                .is_some_and(|document| document.remove_binding(line_index))
        {
            self.status = None;
        }
    }

    fn test_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, down: &HashSet<String>) {
        let Some(document) = self.document.as_ref() else {
            empty_state(ui);
            return;
        };
        let conflicts = document.conflict_groups();
        let conflict_map = conflict_lookup(&conflicts);
        let previous_hover = self.hovered_conflict.clone();
        let mut next_hover = None;
        let player_one = document.keyboard_bindings(1);
        let player_two = document.keyboard_bindings(2);
        let player_one_active = document.active_actions(1, down);
        let player_two_active = document.active_actions(2, down);
        let player_one_macros = macro_labels(document, 1);
        let player_two_macros = macro_labels(document, 2);

        let configured_keys = player_one
            .iter()
            .chain(player_two.iter())
            .map(|binding| binding.source.clone())
            .collect::<HashSet<_>>();
        let mut pressed = down
            .iter()
            .filter(|key| configured_keys.contains(*key))
            .map(|key| key_label(key))
            .collect::<Vec<_>>();
        pressed.sort();
        pressed.dedup();
        let content_width = ui.available_width();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(content_width);
            egui::Frame::new()
                .fill(if pressed.is_empty() { CARD } else { ACTIVE_DIM })
                .stroke(Stroke::new(
                    1.0,
                    if pressed.is_empty() {
                        ACCENT_DIM
                    } else {
                        ACTIVE
                    },
                ))
                .corner_radius(10)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.set_min_height(38.0);
                    if pressed.is_empty() {
                        ui.label(RichText::new("Press any configured keyboard key").strong());
                        ui.label(
                            RichText::new(
                                "Actions light up live; simultaneous inputs are supported.",
                            )
                            .small()
                            .color(TEXT_MUTED),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!("Pressed: {}", pressed.join("  +  ")))
                                .strong()
                                .color(ACTIVE),
                        );
                    }
                });
            ui.add_space(12.0);

            ui.columns(2, |columns| {
                test_player(
                    &mut columns[0],
                    TestPlayerView {
                        player: 1,
                        bindings: &player_one,
                        active_actions: &player_one_active,
                        macro_labels: &player_one_macros,
                        conflict_map: &conflict_map,
                        active_hover: previous_hover.as_deref(),
                    },
                    &mut next_hover,
                );
                test_player(
                    &mut columns[1],
                    TestPlayerView {
                        player: 2,
                        bindings: &player_two,
                        active_actions: &player_two_active,
                        macro_labels: &player_two_macros,
                        conflict_map: &conflict_map,
                        active_hover: previous_hover.as_deref(),
                    },
                    &mut next_hover,
                );
            });
        });

        if next_hover != self.hovered_conflict {
            self.hovered_conflict = next_hover;
            ctx.request_repaint();
        }
    }

    fn capture_window(&mut self, ctx: &egui::Context) {
        let Some(target) = self.capture.clone() else {
            return;
        };
        let title = match &target {
            CaptureTarget::Replace { label, .. } => label.clone(),
            CaptureTarget::Add { player, action } => {
                format!("Player {player} · {}", action_label(action).0)
            }
        };
        let mut cancel = false;
        let mut remove = false;

        egui::Window::new("Press a key")
            .id(egui::Id::new("capture_key_window"))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(390.0, 185.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(title).strong().color(ACCENT));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Press the keyboard key you want to assign").size(16.0));
                    ui.label(
                        RichText::new("Esc cancels · Numpad keys require Num Lock")
                            .small()
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                        if matches!(target, CaptureTarget::Replace { .. })
                            && ui.button("Remove binding").clicked()
                        {
                            remove = true;
                        }
                    });
                });
            });

        if cancel {
            self.capture = None;
        }
        if remove {
            if let CaptureTarget::Replace { line_index, .. } = target
                && let Some(document) = self.document.as_mut()
            {
                document.remove_binding(line_index);
                self.status = None;
            }
            self.capture = None;
        }
    }
}

impl eframe::App for BindStudio {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint_after(Duration::from_millis(16));
        self.handle_dropped_files(&ctx);

        let snapshot = self.scanner.poll();
        self.handle_capture(&snapshot);

        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            self.save();
        }

        self.top_panel(ui);
        self.navigation_and_conflicts(ui);

        egui::Frame::new()
            .fill(BG)
            .inner_margin(18.0)
            .show(ui, |ui| match self.screen {
                Screen::Configure => self.configure_view(ui, &ctx),
                Screen::Test => self.test_view(ui, &ctx, &snapshot.down),
            });

        self.capture_window(&ctx);
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(12, 15, 22);
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(11.0, 6.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn tab_button(ui: &mut egui::Ui, screen: &mut Screen, target: Screen, label: &str) {
    let selected = *screen == target;
    let button = egui::Button::new(RichText::new(label).strong().color(if selected {
        Color32::WHITE
    } else {
        TEXT_MUTED
    }))
    .fill(if selected {
        ACCENT_DIM
    } else {
        Color32::TRANSPARENT
    })
    .stroke(if selected {
        Stroke::new(1.0, ACCENT)
    } else {
        Stroke::NONE
    });
    if ui.add(button).clicked() {
        *screen = target;
    }
}

fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(90.0);
        ui.label(RichText::new("No button config loaded").size(22.0).strong());
        ui.add_space(8.0);
        ui.label(
            RichText::new("Browse for button_config.ini, paste its path above, or drop it here.")
                .color(TEXT_MUTED),
        );
    });
}

fn browse_start_directory(path_text: &str) -> Option<PathBuf> {
    let path = PathBuf::from(path_text.trim().trim_matches('"'));
    if path.is_dir() {
        return Some(path);
    }
    if path.is_file() {
        return path.parent().map(PathBuf::from);
    }
    path.ancestors()
        .skip(1)
        .find(|ancestor| ancestor.is_dir())
        .map(PathBuf::from)
}

fn conflict_lookup(groups: &[ConflictGroup]) -> HashMap<usize, String> {
    groups
        .iter()
        .flat_map(|group| {
            group
                .members
                .iter()
                .map(|binding| (binding.line_index, group.key.clone()))
        })
        .collect()
}

fn ordered_actions(bindings: &[Binding]) -> Vec<String> {
    const STANDARD: &[&str] = &[
        "Up", "Down", "Left", "Right", "Atk1", "Atk2", "Atk3", "Atk4", "Atk6", "Atk8", "Atk9",
        "Atk10", "Start", "Select",
    ];
    let mut actions = STANDARD
        .iter()
        .map(|action| (*action).to_owned())
        .collect::<Vec<_>>();
    let mut extras = bindings
        .iter()
        .map(|binding| binding.action.clone())
        .filter(|action| !STANDARD.contains(&action.as_str()))
        .collect::<Vec<_>>();
    extras.sort();
    extras.dedup();
    actions.extend(extras);
    actions
}

fn grouped_actions(actions: &[String]) -> Vec<(&'static str, Vec<String>)> {
    let movement = actions
        .iter()
        .filter(|action| matches!(action.as_str(), "Up" | "Down" | "Left" | "Right"))
        .cloned()
        .collect::<Vec<_>>();
    let combat = actions
        .iter()
        .filter(|action| action.starts_with("Atk"))
        .cloned()
        .collect::<Vec<_>>();
    let utility = actions
        .iter()
        .filter(|action| {
            !matches!(action.as_str(), "Up" | "Down" | "Left" | "Right")
                && !action.starts_with("Atk")
        })
        .cloned()
        .collect::<Vec<_>>();
    vec![
        ("MOVEMENT", movement),
        ("COMBAT", combat),
        ("UTILITY", utility),
    ]
}

fn action_label(action: &str) -> (&str, &str) {
    match action {
        "Up" => ("Up", "Jump / up"),
        "Down" => ("Down", "Crouch / down"),
        "Left" => ("Left", "Move left"),
        "Right" => ("Right", "Move right"),
        "Atk1" => ("A · Light", "Atk1"),
        "Atk2" => ("B · Medium", "Atk2"),
        "Atk3" => ("C · Heavy", "Atk3"),
        "Atk4" => ("F · Flow", "Atk4"),
        "Atk6" => ("Macro 0", "Atk6"),
        "Atk8" => ("Macro 1", "Atk8"),
        "Atk9" => ("Macro 2", "Atk9"),
        "Atk10" => ("Macro 3", "Atk10"),
        "Start" => ("Start", "Menu confirm"),
        "Select" => ("Select", "Menu secondary"),
        other => (other, "Custom action"),
    }
}

struct TestPlayerView<'a> {
    player: u8,
    bindings: &'a [Binding],
    active_actions: &'a HashSet<String>,
    macro_labels: &'a HashMap<String, String>,
    conflict_map: &'a HashMap<usize, String>,
    active_hover: Option<&'a str>,
}

fn test_player(ui: &mut egui::Ui, view: TestPlayerView<'_>, next_hover: &mut Option<String>) {
    let TestPlayerView {
        player,
        bindings,
        active_actions,
        macro_labels,
        conflict_map,
        active_hover,
    } = view;
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, Color32::from_rgb(46, 53, 70)))
        .corner_radius(12)
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("PLAYER {player}"))
                    .size(17.0)
                    .strong()
                    .color(if player == 1 {
                        ACCENT
                    } else {
                        Color32::from_rgb(142, 166, 255)
                    }),
            );
            ui.add_space(5.0);

            let width = ui.available_width().max(420.0);
            let (canvas, _) =
                ui.allocate_exact_size(egui::vec2(width, 265.0), egui::Sense::hover());
            let painter = ui.painter_at(canvas);

            let pad_center = egui::pos2(canvas.left() + width * 0.23, canvas.top() + 125.0);
            let action_center = egui::pos2(canvas.left() + width * 0.73, canvas.top() + 125.0);

            painter.circle_filled(pad_center, 29.0, Color32::from_rgb(13, 16, 23));

            let dpad = [
                (
                    "Up",
                    vec![
                        egui::pos2(pad_center.x - 29.0, pad_center.y - 10.0),
                        egui::pos2(pad_center.x - 41.0, pad_center.y - 74.0),
                        egui::pos2(pad_center.x + 41.0, pad_center.y - 74.0),
                        egui::pos2(pad_center.x + 29.0, pad_center.y - 10.0),
                    ],
                    egui::pos2(pad_center.x, pad_center.y - 53.0),
                    egui::Rect::from_min_max(
                        egui::pos2(pad_center.x - 43.0, pad_center.y - 76.0),
                        egui::pos2(pad_center.x + 43.0, pad_center.y - 8.0),
                    ),
                ),
                (
                    "Down",
                    vec![
                        egui::pos2(pad_center.x - 29.0, pad_center.y + 10.0),
                        egui::pos2(pad_center.x + 29.0, pad_center.y + 10.0),
                        egui::pos2(pad_center.x + 41.0, pad_center.y + 74.0),
                        egui::pos2(pad_center.x - 41.0, pad_center.y + 74.0),
                    ],
                    egui::pos2(pad_center.x, pad_center.y + 53.0),
                    egui::Rect::from_min_max(
                        egui::pos2(pad_center.x - 43.0, pad_center.y + 8.0),
                        egui::pos2(pad_center.x + 43.0, pad_center.y + 76.0),
                    ),
                ),
                (
                    "Left",
                    vec![
                        egui::pos2(pad_center.x - 10.0, pad_center.y - 29.0),
                        egui::pos2(pad_center.x - 74.0, pad_center.y - 41.0),
                        egui::pos2(pad_center.x - 74.0, pad_center.y + 41.0),
                        egui::pos2(pad_center.x - 10.0, pad_center.y + 29.0),
                    ],
                    egui::pos2(pad_center.x - 42.0, pad_center.y),
                    egui::Rect::from_min_max(
                        egui::pos2(pad_center.x - 76.0, pad_center.y - 43.0),
                        egui::pos2(pad_center.x - 8.0, pad_center.y + 43.0),
                    ),
                ),
                (
                    "Right",
                    vec![
                        egui::pos2(pad_center.x + 10.0, pad_center.y - 29.0),
                        egui::pos2(pad_center.x + 74.0, pad_center.y - 41.0),
                        egui::pos2(pad_center.x + 74.0, pad_center.y + 41.0),
                        egui::pos2(pad_center.x + 10.0, pad_center.y + 29.0),
                    ],
                    egui::pos2(pad_center.x + 42.0, pad_center.y),
                    egui::Rect::from_min_max(
                        egui::pos2(pad_center.x + 8.0, pad_center.y - 43.0),
                        egui::pos2(pad_center.x + 76.0, pad_center.y + 43.0),
                    ),
                ),
            ];

            for (action, points, label_pos, hit_rect) in dpad {
                let action_bindings = bindings_for_action(bindings, action);
                let visual = control_visual(
                    action,
                    &action_bindings,
                    active_actions,
                    conflict_map,
                    active_hover,
                    Color32::from_rgb(55, 104, 116),
                );
                painter.add(egui::Shape::convex_polygon(
                    points,
                    visual.fill,
                    visual.stroke,
                ));
                paint_control_text(
                    &painter,
                    label_pos,
                    action.to_ascii_uppercase().as_str(),
                    &binding_names(&action_bindings),
                    visual.text,
                    12.0,
                );
                let response = ui.interact(
                    hit_rect,
                    egui::Id::new(("test_dpad", player, action)),
                    egui::Sense::hover(),
                );
                register_conflict_hover(response, visual.conflict_key, next_hover);
            }

            let action_buttons = [
                ("Atk2", "B", egui::vec2(0.0, -72.0)),
                ("Atk1", "A", egui::vec2(-73.0, 0.0)),
                ("Atk3", "C", egui::vec2(73.0, 0.0)),
                ("Atk4", "F", egui::vec2(0.0, 72.0)),
            ];
            for (action, letter, offset) in action_buttons {
                let center = action_center + offset;
                let action_bindings = bindings_for_action(bindings, action);
                let visual = control_visual(
                    action,
                    &action_bindings,
                    active_actions,
                    conflict_map,
                    active_hover,
                    Color32::from_rgb(65, 122, 148),
                );
                painter.circle_filled(center + egui::vec2(0.0, 5.0), 42.0, Color32::BLACK);
                painter.circle_filled(center, 42.0, visual.fill);
                painter.circle_stroke(center, 42.0, visual.stroke);
                paint_control_text(
                    &painter,
                    center,
                    letter,
                    &binding_names(&action_bindings),
                    visual.text,
                    21.0,
                );
                let response = ui.interact(
                    egui::Rect::from_center_size(center, egui::vec2(86.0, 86.0)),
                    egui::Id::new(("test_action", player, action)),
                    egui::Sense::hover(),
                );
                register_conflict_hover(response, visual.conflict_key, next_hover);
            }

            ui.separator();
            ui.label(
                RichText::new("EXTRA / MENU")
                    .small()
                    .strong()
                    .color(TEXT_MUTED),
            );
            ui.horizontal_wrapped(|ui| {
                for action in ["Atk6", "Atk8", "Atk9", "Atk10", "Start", "Select"] {
                    let action_bindings = bindings_for_action(bindings, action);
                    if action_bindings.is_empty() {
                        continue;
                    }
                    let visual = control_visual(
                        action,
                        &action_bindings,
                        active_actions,
                        conflict_map,
                        active_hover,
                        CARD_HOVER,
                    );
                    let combo = macro_labels.get(action).map(String::as_str).unwrap_or("—");
                    let label = format!(
                        "{}  {}  {}",
                        action_label(action).0,
                        combo,
                        binding_names(&action_bindings)
                    );
                    let response = ui.add(
                        egui::Button::new(RichText::new(label).color(visual.text))
                            .fill(visual.fill)
                            .stroke(visual.stroke),
                    );
                    register_conflict_hover(response, visual.conflict_key, next_hover);
                }
            });
        });
}

struct ControlVisual {
    fill: Color32,
    stroke: Stroke,
    text: Color32,
    conflict_key: Option<String>,
}

fn bindings_for_action<'a>(bindings: &'a [Binding], action: &str) -> Vec<&'a Binding> {
    bindings
        .iter()
        .filter(|binding| binding.action == action)
        .collect()
}

fn binding_names(bindings: &[&Binding]) -> String {
    if bindings.is_empty() {
        return "UNBOUND".to_owned();
    }
    bindings
        .iter()
        .map(|binding| key_label(&binding.source))
        .collect::<Vec<_>>()
        .join(" / ")
}

fn control_visual(
    action: &str,
    bindings: &[&Binding],
    active_actions: &HashSet<String>,
    conflict_map: &HashMap<usize, String>,
    active_hover: Option<&str>,
    idle_fill: Color32,
) -> ControlVisual {
    let pressed = active_actions.contains(action);
    let conflict_key = bindings
        .iter()
        .find_map(|binding| conflict_map.get(&binding.line_index))
        .cloned();
    let conflict_hot = conflict_key
        .as_deref()
        .is_some_and(|key| active_hover == Some(key));

    let (fill, stroke, text) = if pressed {
        (ACTIVE_DIM, Stroke::new(2.5, ACTIVE), ACTIVE)
    } else if conflict_hot {
        (
            CONFLICT_DIM,
            Stroke::new(2.5, CONFLICT),
            Color32::from_rgb(255, 220, 167),
        )
    } else if conflict_key.is_some() {
        (idle_fill, Stroke::new(2.0, CONFLICT), Color32::WHITE)
    } else {
        (
            idle_fill,
            Stroke::new(1.5, Color32::from_rgb(121, 171, 188)),
            Color32::WHITE,
        )
    };

    ControlVisual {
        fill,
        stroke,
        text,
        conflict_key,
    }
}

fn macro_labels(document: &ButtonConfig, player: u8) -> HashMap<String, String> {
    ["Atk6", "Atk8", "Atk9", "Atk10"]
        .into_iter()
        .filter_map(|action| {
            document
                .macro_inputs_for_action(player, action)
                .map(|inputs| {
                    let readable = inputs
                        .iter()
                        .map(|input| match input.as_str() {
                            "Atk1" => "A",
                            "Atk2" => "B",
                            "Atk3" => "C",
                            "Atk4" => "F",
                            "Up" => "Up",
                            "Down" => "Down",
                            "Left" => "Left",
                            "Right" => "Right",
                            other => other,
                        })
                        .collect::<Vec<_>>()
                        .join("+");
                    (action.to_owned(), readable)
                })
        })
        .collect()
}

fn paint_control_text(
    painter: &egui::Painter,
    center: egui::Pos2,
    name: &str,
    key: &str,
    color: Color32,
    name_size: f32,
) {
    painter.text(
        center + egui::vec2(0.0, -8.0),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(name_size),
        color,
    );
    painter.text(
        center + egui::vec2(0.0, 12.0),
        egui::Align2::CENTER_CENTER,
        key,
        egui::FontId::proportional(9.5),
        color,
    );
}

fn register_conflict_hover(
    response: egui::Response,
    conflict_key: Option<String>,
    next_hover: &mut Option<String>,
) {
    if response.hovered()
        && let Some(key) = conflict_key
    {
        *next_hover = Some(key);
        response.on_hover_text("This key triggers more than one binding");
    }
}

fn main() -> eframe::Result {
    let startup_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([820.0, 600.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Avatar Bind Studio",
        options,
        Box::new(move |creation_context| {
            Ok(Box::new(BindStudio::new(
                &creation_context.egui_ctx,
                startup_path,
            )))
        }),
    )
}

#[cfg(test)]
mod ui_tests {
    use super::*;

    #[test]
    fn browse_starts_from_selected_files_parent() {
        let directory = std::env::temp_dir();
        let file = directory.join(format!("avatar-bind-browse-{}.ini", std::process::id()));
        std::fs::write(&file, "ButtonConfigSet 9\n").unwrap();
        assert_eq!(
            browse_start_directory(&file.display().to_string()),
            Some(directory)
        );
        std::fs::remove_file(file).unwrap();
    }

    #[test]
    fn browse_uses_nearest_existing_parent_for_new_paths() {
        let directory = std::env::temp_dir();
        let file = directory
            .join("folder-that-does-not-exist")
            .join("button_config.ini");
        assert_eq!(
            browse_start_directory(&file.display().to_string()),
            Some(directory)
        );
    }
}
