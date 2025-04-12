use std::collections::HashSet;

use egui::TextWrapMode;
use egui_extras::{Column, TableBuilder};

use crate::replay::{KonohaDifficulty, Mode, ReplayStore};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ManagerUI {
    selected_tab: Tab,
    selected_mode: Mode,
    selected_rows: HashSet<usize>,
    replay_store: ReplayStore,
}

#[derive(PartialEq, serde::Deserialize)]
enum Tab {
    Game,
    Backup,
    Tap,
}

impl Default for ManagerUI {
    fn default() -> Self {
        Self {
            selected_tab: Tab::Game,
            selected_mode: Mode::Normal,
            replay_store: Default::default(),
            selected_rows: Default::default(),
        }
    }
}

impl ManagerUI {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Self {
            replay_store: ReplayStore::new(),
            ..Default::default()
        }
    }

    fn show_table(&mut self, ui: &mut egui::Ui) {
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        //TODO can we remove the clone?
        let replays = match self.selected_mode {
            Mode::Marathon => self.replay_store.marathon.clone(),
            Mode::Master => self.replay_store.master.clone(),
            Mode::Normal => self.replay_store.normal.clone(),
            Mode::Konoha(_) => self.replay_store.konoha.clone(),
            Mode::Shiranui(_, _) => self.replay_store.shiranui.clone(),
            Mode::Asuka => self.replay_store.asuka.clone(),
            Mode::Versus => self.replay_store.pvp.clone(),
        };

        let mut table = TableBuilder::new(ui)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .striped(true)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::remainder())
            .sense(egui::Sense::click())
            .min_scrolled_height(0.0)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Row");
                });
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Options");
                });
                header.col(|ui| {
                    ui.strong("Playtime");
                });
                header.col(|ui| {
                    ui.strong("Score");
                });
                header.col(|ui| {
                    ui.strong("Seed");
                });
                header.col(|ui| {
                    ui.strong("Date");
                });
            })
            .body(|mut body| {
                body.rows(text_height, replays.len(), |mut row| {
                    let row_index = row.index();
                    row.set_selected(self.selected_rows.contains(&row_index));
                    // Useless in Normal/Marathon
                    // self.replay_store.normal.get(row_index).map(|replay| {
                    //     ui.label(replay.rule.to_string());
                    // });
                    if let Some(ref replay) = replays.get(row_index) {
                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        // row.col(|ui| ui.label(format!("{}", replay.steamid)));
                        row.col(|ui| {
                            ui.label(replay.steamid.to_string());
                        });
                        row.col(|ui| {
                            ui.label(replay.level.to_string());
                        });
                        row.col(|ui| {
                            //                            ui.label(replay.modifiers.to_string());
                        });
                        row.col(|ui| {
                            //TODO formatting function I missed somewhere?
                            let secs = replay.time.as_secs();
                            let millis = replay.time.subsec_millis();
                            let (first, second) = (millis / 100, millis % 10);
                            let mins = secs / 60;
                            let secs = secs % 60;
                            ui.label(format!("{mins:0>2}'{secs:0>2}\"{first}{second}"));
                            //ui.label(replay.time.format());
                        });
                        row.col(|ui| {
                            ui.label(replay.score.to_string());
                        });
                        row.col(|ui| {
                            ui.label(replay.seed.to_string());
                        });
                        row.col(|ui| {
                            ui.label(replay.played_at.to_string());
                        });
                    }

                    self.toggle_row_selection(row_index, &row.response());
                });
            });
        //.max_scroll_height(400);
    }
    fn toggle_row_selection(&mut self, row_index: usize, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selected_rows.contains(&row_index) {
                self.selected_rows.remove(&row_index);
            } else {
                self.selected_rows.insert(row_index);
            }
        }
    }
}
impl eframe::App for ManagerUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                //egui::widgets::global_theme_preference_buttons(ui);
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Game, "In Game");
                ui.selectable_value(&mut self.selected_tab, Tab::Backup, "In Backup");
                ui.selectable_value(&mut self.selected_tab, Tab::Tap, "On TheAbsolute.Plus");
            });

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_mode, Mode::Normal, "Normal");
                ui.selectable_value(&mut self.selected_mode, Mode::Marathon, "Marathon");
                ui.selectable_value(&mut self.selected_mode, Mode::Master, "Master");
                ui.selectable_value(
                    &mut self.selected_mode,
                    Mode::Konoha(KonohaDifficulty::Easy),
                    "Konoha",
                );
                ui.selectable_value(&mut self.selected_mode, Mode::Shiranui(0, 0), "Shiranui");
                ui.selectable_value(&mut self.selected_mode, Mode::Asuka, "Asuka");
                ui.selectable_value(&mut self.selected_mode, Mode::Versus, "Versus");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| self.show_table(ui));
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label("Test");
        });
    }
}
