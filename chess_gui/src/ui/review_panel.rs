//! Review side-panel rendering.

use super::*;

impl ChessApp {
    // ---- Review side panel ----

    // ---- Review side panel ----
    pub(super) fn draw_review_side_panel(&mut self, ctx: &egui::Context) {
        const PANEL_TITLE_SIZE: f32 = 13.0;
        const PANEL_TEXT_SIZE: f32 = 12.0;
        const PANEL_META_SIZE: f32 = 11.0;
        egui::SidePanel::right("review_panel")
            .min_width(320.0)
            .default_width(390.0)
            .max_width(560.0)
            .resizable(true)
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(10.0))
            .show(ctx, |ui| {
                // Tab Header
                ui.horizontal(|ui| {
                    let mut active_tab = self
                        .review_state
                        .as_ref()
                        .map(|r| r.active_tab)
                        .unwrap_or(ReviewTab::Report);
                    let tab_width = ui.available_width() / 2.0 - 4.0;

                    let report_color = if active_tab == ReviewTab::Report {
                        Color32::from_rgb(60, 70, 80)
                    } else {
                        Color32::from_rgb(30, 40, 50)
                    };
                    if ui
                        .add_sized(
                            [tab_width, 36.0],
                            egui::Button::new(egui::RichText::new("Report").size(16.0))
                                .fill(report_color),
                        )
                        .clicked()
                    {
                        active_tab = ReviewTab::Report;
                    }

                    let analysis_color = if active_tab == ReviewTab::Analysis {
                        Color32::from_rgb(60, 70, 80)
                    } else {
                        Color32::from_rgb(30, 40, 50)
                    };
                    if ui
                        .add_sized(
                            [tab_width, 36.0],
                            egui::Button::new(egui::RichText::new("Analysis").size(16.0))
                                .fill(analysis_color),
                        )
                        .clicked()
                    {
                        active_tab = ReviewTab::Analysis;
                    }

                    if let Some(ref mut r) = self.review_state {
                        r.active_tab = active_tab;
                    }
                });
                ui.add_space(8.0);

                let mut switch_variation_to: Option<Option<u32>> = None;
                if let Some(ref r) = self.review_state {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(23, 30, 40))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(58, 76, 100)))
                        .inner_margin(8.0)
                        .rounding(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Lines")
                                        .size(PANEL_TITLE_SIZE)
                                        .color(TEXT_SECONDARY),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        egui::Frame::none()
                                            .fill(Color32::from_rgb(18, 24, 33))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(52, 69, 91)))
                                            .rounding(5.0)
                                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Active: {}",
                                                        r.active_variation_name()
                                                    ))
                                                    .size(PANEL_META_SIZE)
                                                    .color(TEXT_MUTED),
                                                );
                                            });
                                    },
                                );
                            });
                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .id_salt("review_lines_tree")
                                .max_height(160.0)
                                .show(ui, |ui| {
                                    let main_selected = r.active_variation_id.is_none();
                                    ui.horizontal(|ui| {
                                        let rail = if main_selected {
                                            Color32::from_rgb(49, 211, 176)
                                        } else {
                                            Color32::from_rgb(66, 84, 107)
                                        };
                                        let (rail_rect, _) = ui.allocate_exact_size(
                                            Vec2::new(3.0, 26.0),
                                            Sense::hover(),
                                        );
                                        ui.painter().rect_filled(rail_rect, 2.0, rail);
                                        let row = egui::Frame::none()
                                            .fill(if main_selected {
                                                Color32::from_rgb(34, 52, 70)
                                            } else {
                                                Color32::from_rgb(24, 33, 44)
                                            })
                                            .stroke(Stroke::new(
                                                1.0,
                                                if main_selected {
                                                    Color32::from_rgb(66, 160, 204)
                                                } else {
                                                    Color32::from_rgb(51, 67, 88)
                                                },
                                            ))
                                            .rounding(6.0)
                                            .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("Main Line")
                                                            .size(PANEL_TEXT_SIZE)
                                                            .color(TEXT_PRIMARY)
                                                            .strong(),
                                                    );
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "{} ply",
                                                                    r.main_records.len()
                                                                ))
                                                                .size(PANEL_META_SIZE)
                                                                .color(TEXT_MUTED),
                                                            );
                                                        },
                                                    );
                                                });
                                            })
                                            .response
                                            .interact(Sense::click())
                                            .on_hover_text("Original game line");

                                        if row.clicked() {
                                            switch_variation_to = Some(None);
                                        }
                                    });
                                    ui.add_space(2.0);

                                    for (id, depth) in r.variation_tree() {
                                        if let Some(var) = r.variations.iter().find(|v| v.id == id)
                                        {
                                            let selected = r.active_variation_id == Some(id);
                                            ui.horizontal(|ui| {
                                                ui.add_space(depth as f32 * 14.0 + 5.0);
                                                let rail = if selected {
                                                    Color32::from_rgb(81, 184, 255)
                                                } else {
                                                    Color32::from_rgb(72, 90, 114)
                                                };
                                                let (rail_rect, _) = ui.allocate_exact_size(
                                                    Vec2::new(3.0, 24.0),
                                                    Sense::hover(),
                                                );
                                                ui.painter().rect_filled(rail_rect, 2.0, rail);

                                                let row = egui::Frame::none()
                                                    .fill(if selected {
                                                        Color32::from_rgb(31, 47, 67)
                                                    } else {
                                                        Color32::from_rgb(22, 30, 41)
                                                    })
                                                    .stroke(Stroke::new(
                                                        1.0,
                                                        if selected {
                                                            Color32::from_rgb(80, 150, 210)
                                                        } else {
                                                            Color32::from_rgb(46, 63, 82)
                                                        },
                                                    ))
                                                    .rounding(6.0)
                                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "└ {}",
                                                                    var.name
                                                                ))
                                                                .size(PANEL_TEXT_SIZE)
                                                                .color(if selected {
                                                                    TEXT_PRIMARY
                                                                } else {
                                                                    TEXT_SECONDARY
                                                                })
                                                                .strong(),
                                                            );
                                                            ui.with_layout(
                                                                egui::Layout::right_to_left(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.label(
                                                                        egui::RichText::new(
                                                                            format!(
                                                                                "A{} · {} ply",
                                                                                var.anchor_index,
                                                                                var.records.len()
                                                                            ),
                                                                        )
                                                                        .size(PANEL_META_SIZE)
                                                                        .color(TEXT_MUTED),
                                                                    );
                                                                },
                                                            );
                                                        });
                                                    })
                                                    .response
                                                    .interact(Sense::click())
                                                    .on_hover_text(format!(
                                                        "branch ply {} • cursor {}",
                                                        var.anchor_index, var.cursor_index
                                                    ));

                                                if row.clicked() {
                                                    switch_variation_to = Some(Some(id));
                                                }
                                            });
                                        }
                                    }
                                });
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("Known positions are reused from cache.")
                                    .size(PANEL_META_SIZE)
                                    .color(TEXT_MUTED),
                            );
                        });
                    ui.add_space(8.0);
                }

                let active_tab = self
                    .review_state
                    .as_ref()
                    .map(|r| r.active_tab)
                    .unwrap_or(ReviewTab::Report);
                if active_tab != ReviewTab::Analysis {
                    self.review_hover_top_line = None;
                }

                // Content area
                egui::ScrollArea::vertical()
                    .id_salt("review_scroll")
                    .max_height(ui.available_height() - 40.0)
                    .show(ui, |ui| match active_tab {
                        ReviewTab::Report => self.draw_review_report_tab(ctx, ui),
                        ReviewTab::Analysis => self.draw_review_analysis_tab(ctx, ui),
                    });

                if let Some(target) = switch_variation_to
                    && let Some(ref mut r) = self.review_state
                {
                    match target {
                        None => r.switch_to_main_line(),
                        Some(id) => r.switch_to_variation(id),
                    }
                }

                // Navigation at the bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let can_prev = self
                        .review_state
                        .as_ref()
                        .map(|r| r.current_index > 0)
                        .unwrap_or(false);
                    let can_next = self
                        .review_state
                        .as_ref()
                        .map(|r| r.current_index + 1 < r.total_positions())
                        .unwrap_or(false);

                    ui.add_space(8.0);

                    // Bottom actions row: progress on the left, actions on the right.
                    ui.horizontal(|ui| {
                        if let Some(ref r) = self.review_state {
                            let total = r.total_positions();
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}/{}",
                                    r.current_index,
                                    if total > 0 { total - 1 } else { 0 }
                                ))
                                .size(13.0)
                                .color(TEXT_SECONDARY),
                            );
                        } else {
                            ui.label(egui::RichText::new("0/0").size(13.0).color(TEXT_SECONDARY));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_sized(
                                    [78.0, 30.0],
                                    egui::Button::new(egui::RichText::new("Back").size(13.0)),
                                )
                                .clicked()
                            {
                                self.app_mode = AppMode::Playing;
                                self.review_state = None;
                                self.review_hover_top_line = None;
                                self.dragging = None;
                            }
                            if ui
                                .add_sized(
                                    [98.0, 30.0],
                                    egui::Button::new(egui::RichText::new("Copy PGN").size(13.0)),
                                )
                                .clicked()
                            {
                                self.do_pgn_copy();
                            }
                        });
                    });

                    if let Some(t) = self.pgn_copied_msg {
                        if t.elapsed().as_secs() < 3 {
                            ui.horizontal(|ui| {
                                ui.colored_label(ACCENT_GREEN, "Copied!");
                            });
                        } else {
                            self.pgn_copied_msg = None;
                        }
                    }

                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(Color32::from_rgb(22, 27, 34))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(49, 62, 81)))
                        .rounding(7.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                let spacing = 8.0;
                                ui.spacing_mut().item_spacing.x = spacing;
                                let btn_w = (ui.available_width() - spacing * 3.0) / 4.0;
                                let btn_size = Vec2::new(btn_w, 36.0);
                                if ui
                                    .add_enabled(
                                        can_prev,
                                        egui::Button::new("|<").min_size(btn_size),
                                    )
                                    .clicked()
                                    && let Some(ref mut r) = self.review_state
                                {
                                    r.go_first();
                                }
                                if ui
                                    .add_enabled(
                                        can_prev,
                                        egui::Button::new("<").min_size(btn_size),
                                    )
                                    .clicked()
                                    && let Some(ref mut r) = self.review_state
                                {
                                    r.go_prev();
                                }
                                if ui
                                    .add_enabled(
                                        can_next,
                                        egui::Button::new(">").min_size(btn_size),
                                    )
                                    .clicked()
                                    && let Some(ref mut r) = self.review_state
                                {
                                    r.go_next();
                                }
                                if ui
                                    .add_enabled(
                                        can_next,
                                        egui::Button::new(">|").min_size(btn_size),
                                    )
                                    .clicked()
                                    && let Some(ref mut r) = self.review_state
                                {
                                    r.go_last();
                                }
                            });
                        });
                });
            });
    }

    fn draw_review_report_tab(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        const REPORT_TITLE_SIZE: f32 = 14.0;
        const REPORT_TEXT_SIZE: f32 = 13.0;
        const REPORT_META_SIZE: f32 = 11.0;
        let mut restore_mainline = false;
        if let Some(ref r) = self.review_state {
            let in_variation = r.is_in_variation();
            if in_variation {
                egui::Frame::none()
                    .fill(Color32::from_rgb(49, 35, 24))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(116, 83, 57)))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Variation line")
                                    .size(REPORT_META_SIZE + 1.0)
                                    .color(Color32::from_rgb(242, 207, 170)),
                            );
                            if ui
                                .add_sized(
                                    [152.0, 28.0],
                                    egui::Button::new(
                                        egui::RichText::new("Return to Main Line")
                                            .size(REPORT_META_SIZE + 1.0),
                                    ),
                                )
                                .clicked()
                            {
                                restore_mainline = true;
                            }
                        });
                    });
                ui.add_space(4.0);
            }

            egui::Frame::none()
                .fill(Color32::from_rgb(24, 34, 28))
                .stroke(Stroke::new(1.0, Color32::from_rgb(58, 130, 86)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(9.0, 7.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("★")
                                .size(18.0)
                                .color(Color32::from_rgb(72, 223, 119))
                                .strong(),
                        );
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(Self::review_best_tip_text(r))
                                    .size(REPORT_TEXT_SIZE)
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(Self::opening_name_from_records(
                                    &r.main_records,
                                ))
                                .size(REPORT_META_SIZE)
                                .color(TEXT_MUTED),
                            );
                        });
                    });
                });
            ui.add_space(8.0);

            if r.analyzing {
                ui.add_space(4.0);
                let progress = r.analysis_progress();
                ui.add(
                    egui::ProgressBar::new(progress)
                        .fill(ACCENT_GREEN)
                        .text("Analyzing..."),
                );
            }

            // Draw accuracy graph
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 25, 30))
                .stroke(Stroke::new(1.0, Color32::from_rgb(50, 60, 70)))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    let rect = ui.allocate_space(Vec2::new(ui.available_width(), 96.0)).1;
                    let painter = ui.painter_at(rect);
                    // Base line (0.0 eval)
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x, rect.center().y),
                            Pos2::new(rect.max.x, rect.center().y),
                        ],
                        Stroke::new(1.0, Color32::from_rgb(80, 90, 100)),
                    );

                    if r.analyses.len() > 1 {
                        let step = rect.width() / (r.analyses.len() as f32 - 1.0).max(1.0);
                        let center_y = rect.center().y;
                        let mut points = Vec::new();
                        for (i, a) in r.analyses.iter().enumerate() {
                            let x = rect.min.x + i as f32 * step;
                            let y = if let Some(a) = a {
                                if !a.lines.is_empty() {
                                    let mut cp = a.lines[0].eval_cp;
                                    if i % 2 != 0 {
                                        cp = -cp;
                                    } // Convert to white POV
                                    // Smoothly map centipawn to chart height while preserving trend direction.
                                    let cp = cp.clamp(-1800, 1800) as f32;
                                    let offset = (cp / 420.0).tanh() * (rect.height() * 0.44);
                                    center_y - offset
                                } else {
                                    let board = &r.snapshots[i];
                                    let color = if i % 2 == 0 {
                                        crate::types::Color::White
                                    } else {
                                        crate::types::Color::Black
                                    };
                                    if board.is_in_check(color) {
                                        let mut cp: f32 = -30000.0;
                                        if i % 2 != 0 {
                                            cp = -cp;
                                        }
                                        let offset = (cp.clamp(-1800.0, 1800.0) / 420.0).tanh()
                                            * (rect.height() * 0.44);
                                        center_y - offset
                                    } else {
                                        center_y
                                    }
                                }
                            } else {
                                center_y
                            };
                            points.push(Pos2::new(x, y));
                        }

                        if points.len() >= 2 {
                            // Weighted smoothing to avoid jagged "raw sample" look.
                            let mut trend = points.clone();
                            for _ in 0..2 {
                                let mut next = trend.clone();
                                for i in 1..trend.len() - 1 {
                                    next[i].y = trend[i - 1].y * 0.22
                                        + trend[i].y * 0.56
                                        + trend[i + 1].y * 0.22;
                                }
                                trend = next;
                            }

                            // Subtle guide lines
                            for frac in [0.25f32, 0.5, 0.75] {
                                let y = rect.min.y + rect.height() * frac;
                                painter.line_segment(
                                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                                    Stroke::new(
                                        1.0,
                                        Color32::from_rgba_unmultiplied(120, 130, 145, 35),
                                    ),
                                );
                            }

                            // Area fill to baseline segment-by-segment (keeps valid convex quads).
                            for i in 0..trend.len().saturating_sub(1) {
                                let p1 = trend[i];
                                let p2 = trend[i + 1];
                                let avg_above = ((p1.y + p2.y) * 0.5) < center_y;
                                let fill = if avg_above {
                                    Color32::from_rgba_unmultiplied(236, 241, 249, 52)
                                } else {
                                    Color32::from_rgba_unmultiplied(122, 146, 179, 44)
                                };
                                painter.add(egui::Shape::convex_polygon(
                                    vec![
                                        Pos2::new(p1.x, center_y),
                                        p1,
                                        p2,
                                        Pos2::new(p2.x, center_y),
                                    ],
                                    fill,
                                    Stroke::NONE,
                                ));
                            }

                            // Trend polyline with soft shadow for clearer direction.
                            for i in 0..trend.len().saturating_sub(1) {
                                let p1 = trend[i];
                                let p2 = trend[i + 1];
                                painter.line_segment(
                                    [p1 + egui::vec2(0.0, 1.0), p2 + egui::vec2(0.0, 1.0)],
                                    Stroke::new(3.2, Color32::from_rgba_unmultiplied(0, 0, 0, 62)),
                                );
                                painter.line_segment(
                                    [p1, p2],
                                    Stroke::new(2.4, Color32::from_rgb(242, 246, 252)),
                                );
                            }

                            // Key points markers
                            let marker_step = (trend.len() / 18).max(1);
                            for (i, p) in trend.iter().enumerate() {
                                if i % marker_step == 0 || i + 1 == trend.len() {
                                    painter.circle_filled(
                                        *p,
                                        2.2,
                                        Color32::from_rgb(242, 246, 252),
                                    );
                                }
                            }
                        }
                    }
                });

            // Accuracies logic
            let compute_accuracy = |is_white: bool| -> f32 {
                let mut total_score = 0.0;
                let mut count = 0;
                for (i, c) in r.classifications.iter().enumerate() {
                    if (i % 2 == 0) == is_white
                        && let Some(class) = c
                    {
                        count += 1;
                        total_score += match class {
                            MoveClassification::Brilliant => 100.0,
                            MoveClassification::Critical => 98.0,
                            MoveClassification::Best => 96.0,
                            MoveClassification::Excellent => 90.0,
                            MoveClassification::Okay => 78.0,
                            MoveClassification::Inaccuracy => 60.0,
                            MoveClassification::Mistake => 35.0,
                            MoveClassification::Blunder => 8.0,
                        };
                    }
                }
                if count == 0 {
                    100.0
                } else {
                    total_score / count as f32
                }
            };

            let white_accuracy = compute_accuracy(true);
            let black_accuracy = compute_accuracy(false);
            let player_color = self.human_color;
            let ai_color = player_color.opposite();
            let player_name = self.human_name_display();
            let ai_name = self.opponent_name_display();
            let player_accuracy = if player_color == Color::White {
                white_accuracy
            } else {
                black_accuracy
            };
            let ai_accuracy = if ai_color == Color::White {
                white_accuracy
            } else {
                black_accuracy
            };
            let player_label = if player_color == Color::White {
                format!("{} (W)", player_name)
            } else {
                format!("{} (B)", player_name)
            };
            let ai_label = if ai_color == Color::White {
                format!("{} (W)", ai_name)
            } else {
                format!("{} (B)", ai_name)
            };

            ui.add_space(7.0);
            ui.label(
                egui::RichText::new("Accuracies")
                    .size(REPORT_TITLE_SIZE)
                    .color(TEXT_SECONDARY),
            );
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 35, 40))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let width = ui.available_width() / 2.0;
                        ui.allocate_ui_with_layout(
                            Vec2::new(width, 68.0),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(&player_label)
                                            .size(REPORT_META_SIZE)
                                            .color(TEXT_MUTED),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{:.1}%", player_accuracy))
                                            .size(30.0)
                                            .color(Color32::WHITE)
                                            .strong(),
                                    );
                                });
                            },
                        );
                        ui.allocate_ui_with_layout(
                            Vec2::new(width, 68.0),
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(&ai_label)
                                            .size(REPORT_META_SIZE)
                                            .color(TEXT_MUTED),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{:.1}%", ai_accuracy))
                                            .size(30.0)
                                            .color(Color32::from_rgb(210, 210, 210))
                                            .strong(),
                                    );
                                });
                            },
                        );
                    });
                });

            ui.add_space(8.0);

            // Move Classifications Grid
            egui::Frame::none()
                .fill(Color32::from_rgb(24, 30, 40))
                .inner_margin(egui::Margin::symmetric(10.0, 9.0))
                .show(ui, |ui| {
                    let mut w_counts = std::collections::HashMap::new();
                    let mut b_counts = std::collections::HashMap::new();
                    for (i, c) in r.classifications.iter().enumerate() {
                        if let Some(class) = c {
                            if i % 2 == 0 {
                                *w_counts.entry(*class).or_insert(0) += 1;
                            } else {
                                *b_counts.entry(*class).or_insert(0) += 1;
                            }
                        }
                    }
                    let (player_counts, ai_counts) = if self.human_color == Color::White {
                        (&w_counts, &b_counts)
                    } else {
                        (&b_counts, &w_counts)
                    };

                    let avail = ui.available_width();
                    egui::Grid::new("classifications_grid")
                        .num_columns(4)
                        .min_col_width(avail / 4.0 - 10.0)
                        .spacing([8.0, 12.0])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Name")
                                    .size(REPORT_META_SIZE)
                                    .color(TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Icon")
                                    .size(REPORT_META_SIZE)
                                    .color(TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new(player_label)
                                    .size(REPORT_META_SIZE)
                                    .color(TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new(ai_label)
                                    .size(REPORT_META_SIZE)
                                    .color(TEXT_SECONDARY),
                            );
                            ui.end_row();

                            for class in [
                                MoveClassification::Brilliant,
                                MoveClassification::Critical,
                                MoveClassification::Best,
                                MoveClassification::Excellent,
                                MoveClassification::Okay,
                                MoveClassification::Inaccuracy,
                                MoveClassification::Mistake,
                                MoveClassification::Blunder,
                            ] {
                                let player_val = player_counts.get(&class).unwrap_or(&0);
                                let ai_val = ai_counts.get(&class).unwrap_or(&0);

                                let rgb = class.color32();
                                let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                let icon_color = color;

                                ui.label(
                                    egui::RichText::new(class.label())
                                        .size(15.0)
                                        .color(color)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new(class.icon())
                                        .size(16.0)
                                        .color(icon_color),
                                );
                                ui.label(
                                    egui::RichText::new(player_val.to_string())
                                        .size(REPORT_TEXT_SIZE)
                                        .color(TEXT_PRIMARY),
                                );
                                ui.label(
                                    egui::RichText::new(ai_val.to_string())
                                        .size(REPORT_TEXT_SIZE)
                                        .color(TEXT_PRIMARY),
                                );
                                ui.end_row();
                            }
                        });
                });
        }

        if restore_mainline && let Some(ref mut r) = self.review_state {
            r.restore_main_line();
        }
    }

    fn draw_review_analysis_tab(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        const ANALYSIS_TITLE_SIZE: f32 = 14.0;
        const ANALYSIS_TEXT_SIZE: f32 = 14.0;
        const ANALYSIS_META_SIZE: f32 = 13.0;
        let mut restore_mainline = false;
        if let Some(ref r) = self.review_state {
            if r.analyzing {
                ui.add_space(4.0);
                let progress = r.analysis_progress();
                ui.add(
                    egui::ProgressBar::new(progress)
                        .fill(ACCENT_GREEN)
                        .text("Analyzing..."),
                );
            }

            let in_variation = r.is_in_variation();
            if in_variation {
                egui::Frame::none()
                    .fill(Color32::from_rgb(49, 35, 24))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(116, 83, 57)))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Variation line")
                                    .size(ANALYSIS_META_SIZE)
                                    .color(Color32::from_rgb(242, 207, 170)),
                            );
                            if ui
                                .add_sized(
                                    [152.0, 28.0],
                                    egui::Button::new(
                                        egui::RichText::new("Return to Main Line")
                                            .size(ANALYSIS_META_SIZE),
                                    ),
                                )
                                .clicked()
                            {
                                restore_mainline = true;
                            }
                        });
                    });
                ui.add_space(4.0);
            }
        }

        if restore_mainline {
            if let Some(ref mut r) = self.review_state {
                r.restore_main_line();
            }
            return;
        }

        ui.add_space(6.0);
        if let Some(ref r) = self.review_state
            && let Some(Some(analysis)) = r.analyses.get(r.current_index)
            && let Some(line) = analysis.lines.first()
        {
            let eval_disp = if line.eval_cp.abs() > 29000 {
                let mate_ply = 30000 - line.eval_cp.abs();
                if line.eval_cp > 0 {
                    format!("M{}", mate_ply)
                } else {
                    format!("-M{}", mate_ply)
                }
            } else {
                format!("{:+.2}", line.eval_cp as f32 / 100.0)
            };

            egui::Frame::none()
                .fill(Color32::from_rgb(29, 36, 47))
                .stroke(Stroke::new(1.0, Color32::from_rgb(58, 74, 96)))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Eval")
                                .size(ANALYSIS_META_SIZE)
                                .color(TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(eval_disp)
                                .size(20.0)
                                .color(ACCENT_GREEN)
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Depth {}", analysis.depth))
                                .size(ANALYSIS_META_SIZE)
                                .color(TEXT_MUTED),
                        );
                        if analysis.verification_passes > 1 {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!(
                                    "Vote {}/{} · Stable {}%",
                                    analysis.verification_passes,
                                    analysis.verification_passes,
                                    analysis.stability_pct
                                ))
                                .size(ANALYSIS_META_SIZE)
                                .color(TEXT_MUTED),
                            );
                        }
                    });
                });
        }

        ui.add_space(8.0);

        let mut chosen_alt_move = None;
        let mut hovered_top_line = None;
        if let Some(ref r) = self.review_state
            && let Some(Some(analysis)) = r.analyses.get(r.current_index)
            && !analysis.lines.is_empty()
        {
            ui.label(
                egui::RichText::new("Top Lines")
                    .size(ANALYSIS_TITLE_SIZE)
                    .color(TEXT_SECONDARY),
            );
            ui.add_space(2.0);
            let current_board = r.current_board().clone();
            let current_color = if r.current_index.is_multiple_of(2) {
                Color::White
            } else {
                Color::Black
            };
            for (i, line) in analysis.lines.iter().enumerate() {
                let eval_str = if line.eval_cp > 29000 {
                    format!("M{}", 30000 - line.eval_cp)
                } else if line.eval_cp < -29000 {
                    format!("-M{}", 30000 + line.eval_cp)
                } else {
                    format!("{:.2}", line.eval_cp as f32 / 100.0)
                };
                let san_text = Board::uci_to_move(&line.best_move_uci)
                    .ok()
                    .map(|mv| pgn::move_to_san(&current_board, &mv, current_color))
                    .filter(|san| !san.is_empty())
                    .unwrap_or_else(|| line.best_move_uci.clone());
                let line_color = match i {
                    0 => Color32::from_rgb(39, 211, 169),
                    1 => Color32::from_rgb(81, 184, 255),
                    _ => Color32::from_rgb(175, 153, 255),
                };
                let row = egui::Frame::none()
                    .fill(Color32::from_rgb(25, 34, 46))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(56, 72, 94)))
                    .inner_margin(egui::Margin::symmetric(9.0, 8.0))
                    .rounding(7.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(
                                    line_color.r(),
                                    line_color.g(),
                                    line_color.b(),
                                    36,
                                ))
                                .stroke(Stroke::new(1.0, line_color))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("#{}", i + 1))
                                            .size(11.0)
                                            .color(line_color)
                                            .strong(),
                                    );
                                });

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(san_text)
                                        .size(ANALYSIS_TEXT_SIZE)
                                        .color(TEXT_PRIMARY)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new(&line.best_move_uci)
                                        .size(ANALYSIS_META_SIZE)
                                        .color(TEXT_MUTED),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    egui::Frame::none()
                                        .fill(Color32::from_rgb(17, 24, 34))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(57, 73, 92)))
                                        .rounding(5.0)
                                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(eval_str)
                                                    .size(ANALYSIS_META_SIZE)
                                                    .color(line_color)
                                                    .strong(),
                                            );
                                        });
                                },
                            );
                        });
                    })
                    .response
                    .interact(Sense::click())
                    .on_hover_text("Click to explore this line");
                if row.hovered() {
                    hovered_top_line = Some(i);
                }
                if row.clicked() {
                    chosen_alt_move = Some(line.best_move_uci.clone());
                }
                ui.add_space(2.0);
            }
        }
        self.review_hover_top_line = hovered_top_line;

        if let Some(uci_mov) = chosen_alt_move
            && let Ok(mv) = Board::uci_to_move(&uci_mov)
            && let Some(ref mut r) = self.review_state
        {
            let current_board = r.current_board().clone();
            let current_color = if r.current_index.is_multiple_of(2) {
                Color::White
            } else {
                Color::Black
            };
            let san = crate::pgn::move_to_san(&current_board, &mv, current_color);
            r.line_apply_existing_or_branch_move(mv, san, current_color);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new("Moves")
                .size(ANALYSIS_TITLE_SIZE)
                .color(TEXT_SECONDARY),
        );

        let move_display: Vec<_> = if let Some(ref review) = self.review_state {
            let mut data = Vec::new();
            let branch_len = review.move_records.len();
            let main_len = review.main_records.len();
            let max_len = branch_len.max(main_len);
            let mut i = 0usize;

            while i < max_len {
                let (w_text, w_color, w_class, w_mainline_only) = if i < branch_len {
                    let w_class = review.classifications.get(i).and_then(|c| *c);
                    let w_color = w_class
                        .map(|c| {
                            let rgb = c.color32();
                            Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                        })
                        .unwrap_or(TEXT_PRIMARY);
                    let w_text = if let Some(class) = w_class {
                        format!("{} {}", review.move_records[i].san, class.icon())
                    } else {
                        review.move_records[i].san.clone()
                    };
                    (w_text, w_color, w_class, false)
                } else if i < main_len {
                    (
                        format!("{} ⤴", review.main_records[i].san),
                        TEXT_SECONDARY,
                        None,
                        true,
                    )
                } else {
                    ("".to_string(), TEXT_MUTED, None, false)
                };

                let black_part = if i + 1 < branch_len {
                    let b_class = review.classifications.get(i + 1).and_then(|c| *c);
                    let b_color = b_class
                        .map(|c| {
                            let rgb = c.color32();
                            Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                        })
                        .unwrap_or(TEXT_PRIMARY);
                    let b_text = if let Some(class) = b_class {
                        format!("{} {}", review.move_records[i + 1].san, class.icon())
                    } else {
                        review.move_records[i + 1].san.clone()
                    };
                    Some((b_text, b_color, b_class, false))
                } else if i + 1 < main_len {
                    Some((
                        format!("{} ⤴", review.main_records[i + 1].san),
                        TEXT_SECONDARY,
                        None,
                        true,
                    ))
                } else {
                    None
                };

                data.push((i, w_text, w_color, w_class, w_mainline_only, black_part));
                i += 2;
            }
            data
        } else {
            Vec::new()
        };

        let mut clicked_index = None;
        let mut clicked_mainline_only = false;
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 5.0)
            .show(ui, |ui| {
                for (i, w_san, w_color, w_class, w_mainline_only, black_part) in &move_display {
                    let move_num = i / 2 + 1;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:>2}.", move_num))
                                .size(ANALYSIS_META_SIZE)
                                .color(TEXT_MUTED)
                                .monospace(),
                        );
                        let w_text = egui::RichText::new(w_san)
                            .size(ANALYSIS_TEXT_SIZE)
                            .color(*w_color)
                            .monospace();
                        let mut w_resp = ui.add(egui::Label::new(w_text).sense(Sense::click()));
                        if let Some(class) = w_class {
                            w_resp = w_resp.on_hover_text(class.label());
                        }
                        if w_resp.clicked() {
                            clicked_index = Some(i + 1);
                            clicked_mainline_only = *w_mainline_only;
                        }

                        if let Some((b_san, b_color, b_class, b_mainline_only)) = black_part {
                            let b_text = egui::RichText::new(b_san)
                                .size(ANALYSIS_TEXT_SIZE)
                                .color(*b_color)
                                .monospace();
                            let mut b_resp = ui.add(egui::Label::new(b_text).sense(Sense::click()));
                            if let Some(class) = b_class {
                                b_resp = b_resp.on_hover_text(class.label());
                            }
                            if b_resp.clicked() {
                                clicked_index = Some(i + 2);
                                clicked_mainline_only = *b_mainline_only;
                            }
                        }
                    });
                }
            });

        if let Some(idx) = clicked_index
            && let Some(ref mut r) = self.review_state
        {
            if clicked_mainline_only {
                r.restore_main_line();
            }
            r.current_index = idx.min(r.total_positions().saturating_sub(1));
        }
    }
}
