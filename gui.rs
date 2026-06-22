use eframe::egui;
use std::collections::{HashMap, HashSet};

use crate::bin::{Bin, Placement};
use crate::evaluator::evaluate;
use crate::generator::generate_instance;
use crate::greedy::{run_greedy, GreedyStrategy, PackingSolution};
use crate::instance::Instance;
use crate::local_search::{run_local_search_swap, LocalSearchStart};
use crate::local_search_geometry::run_geometry_local_search;
use crate::local_search_overlap::{run_overlap_local_search, run_overlap_local_search_with_start};

pub struct PackingApp {
    pub instance: Option<Instance>,
    pub solution: Option<PackingSolution>,
    pub selected_algo: usize,
    pub status: String,

    pub num_rects: usize,
    pub min_size: u32,
    pub max_size: u32,
    pub box_size: u32,

    pub history: Vec<PackingSolution>,
    pub history_index: usize,

    pub tracked_rectangles_input: String,
    pub tracked_rectangles: Vec<usize>,
    pub auto_track_most_changed: bool,
}

impl Default for PackingApp {
    fn default() -> Self {
        Self {
            instance: None,
            solution: None,
            selected_algo: 0,
            status: "Ready".to_string(),
            num_rects: 10,
            min_size: 1,
            max_size: 8,
            box_size: 10,
            history: Vec::new(),
            history_index: 0,
            tracked_rectangles_input: String::new(),
            tracked_rectangles: Vec::new(),
            auto_track_most_changed: true,
        }
    }
}

impl PackingApp {
    fn current_display_solution(&self) -> Option<&PackingSolution> {
        if !self.history.is_empty() {
            self.history.get(self.history_index)
        } else {
            self.solution.as_ref()
        }
    }

    fn previous_display_solution(&self) -> Option<&PackingSolution> {
        if !self.history.is_empty() && self.history_index > 0 {
            self.history.get(self.history_index - 1)
        } else {
            None
        }
    }

    fn is_small_demo_mode(&self) -> bool {
        self.selected_algo >= 2
            && self
                .instance
                .as_ref()
                .map(|i| i.rectangles.len() <= 25)
                .unwrap_or(false)
    }

    fn solution_signature(solution: &PackingSolution) -> String {
        let mut entries: Vec<String> = Vec::new();

        for (bin_index, bin) in solution.bins.iter().enumerate() {
            for p in &bin.placements {
                entries.push(format!(
                    "{}:{}:{}:{}:{}:{}:{}",
                    bin_index, p.rect_id, p.x, p.y, p.width, p.height, p.rotated
                ));
            }
        }

        entries.sort();
        entries.join("|")
    }

    fn should_keep_snapshot(
        &self,
        previous: &PackingSolution,
        current: &PackingSolution,
    ) -> bool {
        let prev_score = evaluate(previous, self.box_size);
        let curr_score = evaluate(current, self.box_size);

        if prev_score.bins_used != curr_score.bins_used {
            return true;
        }

        if prev_score.wasted_area != curr_score.wasted_area {
            return true;
        }

        let prev_sig = Self::solution_signature(previous);
        let curr_sig = Self::solution_signature(current);

        prev_sig != curr_sig
    }

    fn set_history_filtered(&mut self, snapshots: Vec<PackingSolution>) {
        let mut filtered: Vec<PackingSolution> = Vec::new();

        for snapshot in snapshots {
            if let Some(last) = filtered.last() {
                if self.should_keep_snapshot(last, &snapshot) {
                    filtered.push(snapshot);
                }
            } else {
                filtered.push(snapshot);
            }
        }

        if filtered.is_empty() {
            self.history.clear();
            self.history_index = 0;
            self.solution = None;
        } else {
            self.history = filtered;
            self.history_index = self.history.len().saturating_sub(1);
            self.solution = self.current_display_solution().cloned();
        }
    }

    fn set_history_raw(&mut self, snapshots: Vec<PackingSolution>) {
        if snapshots.is_empty() {
            self.history.clear();
            self.history_index = 0;
            self.solution = None;
        } else {
            self.history = snapshots;
            self.history_index = self.history.len().saturating_sub(1);
            self.solution = self.current_display_solution().cloned();
        }
    }

    fn parse_tracked_rectangles(&mut self) {
        let mut ids = Vec::new();

        for part in self.tracked_rectangles_input.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(id) = trimmed.parse::<usize>() {
                ids.push(id);
            }
        }

        ids.sort_unstable();
        ids.dedup();
        self.tracked_rectangles = ids;
    }

    fn find_rect_position(
        solution: &PackingSolution,
        rect_id: usize,
    ) -> Option<(usize, u32, u32, u32, u32, bool)> {
        for (bin_index, bin) in solution.bins.iter().enumerate() {
            for p in &bin.placements {
                if p.rect_id == rect_id {
                    return Some((bin_index, p.x, p.y, p.width, p.height, p.rotated));
                }
            }
        }
        None
    }

    fn placement_map(
        solution: &PackingSolution,
    ) -> HashMap<usize, (usize, u32, u32, u32, u32, bool)> {
        let mut map = HashMap::new();
        for (bin_index, bin) in solution.bins.iter().enumerate() {
            for p in &bin.placements {
                map.insert(
                    p.rect_id,
                    (bin_index, p.x, p.y, p.width, p.height, p.rotated),
                );
            }
        }
        map
    }

    fn changed_rectangles_between(
        previous: &PackingSolution,
        current: &PackingSolution,
    ) -> Vec<usize> {
        let prev = Self::placement_map(previous);
        let curr = Self::placement_map(current);

        let mut ids: HashSet<usize> = HashSet::new();
        ids.extend(prev.keys().copied());
        ids.extend(curr.keys().copied());

        let mut changed = Vec::new();

        for rect_id in ids {
            if prev.get(&rect_id) != curr.get(&rect_id) {
                changed.push(rect_id);
            }
        }

        changed.sort_unstable();
        changed
    }

    fn most_changed_rectangles(&self, top_k: usize) -> Vec<usize> {
        if self.history.len() < 2 {
            return Vec::new();
        }

        let mut scores: HashMap<usize, i32> = HashMap::new();

        for i in 1..self.history.len() {
            let prev = Self::placement_map(&self.history[i - 1]);
            let curr = Self::placement_map(&self.history[i]);

            let mut ids: HashSet<usize> = HashSet::new();
            ids.extend(prev.keys().copied());
            ids.extend(curr.keys().copied());

            for rect_id in ids {
                let prev_pos = prev.get(&rect_id);
                let curr_pos = curr.get(&rect_id);

                if prev_pos != curr_pos {
                    let entry = scores.entry(rect_id).or_insert(0);

                    match (prev_pos, curr_pos) {
                        (Some((pb, px, py, pw, ph, prot)), Some((cb, cx, cy, cw, ch, crot))) => {
                            if pb != cb {
                                *entry += 5;
                            }
                            if px != cx || py != cy {
                                *entry += 1;
                            }
                            if pw != cw || ph != ch || prot != crot {
                                *entry += 2;
                            }
                        }
                        _ => {
                            *entry += 3;
                        }
                    }
                }
            }
        }

        let mut ranked: Vec<(usize, i32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        ranked.into_iter().take(top_k).map(|(id, _)| id).collect()
    }

    fn effective_tracked_rectangles(&self) -> Vec<usize> {
        if !self.tracked_rectangles.is_empty() {
            self.tracked_rectangles.clone()
        } else if self.auto_track_most_changed {
            self.most_changed_rectangles(1)
        } else {
            Vec::new()
        }
    }

    fn highlighted_color(index: usize) -> egui::Color32 {
        const COLORS: [egui::Color32; 8] = [
            egui::Color32::from_rgb(0, 102, 255),
            egui::Color32::from_rgb(255, 99, 132),
            egui::Color32::from_rgb(54, 162, 235),
            egui::Color32::from_rgb(255, 206, 86),
            egui::Color32::from_rgb(75, 192, 192),
            egui::Color32::from_rgb(153, 102, 255),
            egui::Color32::from_rgb(255, 159, 64),
            egui::Color32::from_rgb(0, 200, 120),
        ];
        COLORS[index % COLORS.len()]
    }

    fn normal_rect_color(rect_id: usize) -> egui::Color32 {
        egui::Color32::from_rgb(
            100 + ((rect_id * 53) % 155) as u8,
            100 + ((rect_id * 97) % 155) as u8,
            100 + ((rect_id * 151) % 155) as u8,
        )
    }

    fn faded_rect_color() -> egui::Color32 {
        egui::Color32::from_rgb(130, 130, 130)
    }

    fn build_scattered_solution(instance: &Instance) -> PackingSolution {
        let mut bins: Vec<Bin> = Vec::new();

        for rect in &instance.rectangles {
            let (width, height, rotated) =
                if rect.width <= instance.box_size && rect.height <= instance.box_size {
                    (rect.width, rect.height, false)
                } else if rect.height <= instance.box_size && rect.width <= instance.box_size {
                    (rect.height, rect.width, true)
                } else if rect.height <= instance.box_size {
                    (rect.height, rect.width, true)
                } else {
                    (
                        rect.width.min(instance.box_size),
                        rect.height.min(instance.box_size),
                        false,
                    )
                };

            bins.push(Bin {
                id: bins.len(),
                size: instance.box_size,
                placements: vec![Placement {
                    rect_id: rect.id,
                    x: 0,
                    y: 0,
                    width,
                    height,
                    rotated,
                }],
            });
        }

        PackingSolution { bins }
    }

    fn visual_focus_rectangles(&self) -> Vec<usize> {
        if self.is_small_demo_mode() {
            if let (Some(prev), Some(curr)) =
                (self.previous_display_solution(), self.current_display_solution())
            {
                let changed = Self::changed_rectangles_between(prev, curr);
                if !changed.is_empty() {
                    return changed;
                }
            }
        }

        self.effective_tracked_rectangles()
    }

    fn color_for_rect(&self, rect_id: usize, focus_ids: &[usize]) -> egui::Color32 {
        if self.is_small_demo_mode() {
            if let Some(index) = focus_ids.iter().position(|&id| id == rect_id) {
                Self::highlighted_color(index)
            } else {
                Self::faded_rect_color()
            }
        } else if focus_ids.is_empty() {
            Self::normal_rect_color(rect_id)
        } else if let Some(index) = focus_ids.iter().position(|&id| id == rect_id) {
            Self::highlighted_color(index)
        } else {
            Self::faded_rect_color()
        }
    }

    fn stroke_for_rect(
        &self,
        rect_id: usize,
        is_largest: bool,
        focus_ids: &[usize],
    ) -> egui::Stroke {
        if let Some(index) = focus_ids.iter().position(|&id| id == rect_id) {
            egui::Stroke::new(3.0, Self::highlighted_color(index))
        } else if !self.is_small_demo_mode() && focus_ids.is_empty() && is_largest {
            egui::Stroke::new(2.0, egui::Color32::YELLOW)
        } else {
            egui::Stroke::new(1.0, egui::Color32::BLACK)
        }
    }

    fn should_draw_label(
        &self,
        _rect_id: usize,
        _width_cells: u32,
        _height_cells: u32,
        _focus_ids: &[usize],
    ) -> bool {
        true
    }

    fn draw_bin(
        &self,
        ui: &mut egui::Ui,
        bin_index: usize,
        bin: &Bin,
        focus_ids: &[usize],
    ) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(format!("Bin {}", bin_index))
                    .size(16.0)
                    .strong(),
            );

            let size = 140.0;
            let (response, painter) =
                ui.allocate_painter(egui::vec2(size, size), egui::Sense::hover());

            let rect = response.rect;

            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::WHITE),
            );

            for i in 0..=self.box_size {
                let t = i as f32 / self.box_size as f32;
                let x = rect.left() + t * size;
                let y = rect.top() + t * size;

                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(0.4, egui::Color32::DARK_GRAY),
                );
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(0.4, egui::Color32::DARK_GRAY),
                );
            }

            let largest_area = bin
                .placements
                .iter()
                .map(|p| p.width * p.height)
                .max()
                .unwrap_or(0);

            for p in &bin.placements {
                let x = rect.left_top().x + (p.x as f32 / self.box_size as f32) * size;
                let y = rect.left_top().y + (p.y as f32 / self.box_size as f32) * size;
                let w = (p.width as f32 / self.box_size as f32) * size;
                let h = (p.height as f32 / self.box_size as f32) * size;

                let r = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h));

                let is_largest = p.width * p.height == largest_area;
                let fill = self.color_for_rect(p.rect_id, focus_ids);
                painter.rect_filled(r, 0.0, fill);

                let stroke = self.stroke_for_rect(p.rect_id, is_largest, focus_ids);
                painter.rect_stroke(r, 0.0, stroke);

                if self.should_draw_label(p.rect_id, p.width, p.height, focus_ids) {
                    let text_color = if focus_ids.contains(&p.rect_id) {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };

                    let font_size = if p.width <= 1 || p.height <= 1 {
                        8.0
                    } else if p.width <= 2 || p.height <= 2 {
                        10.0
                    } else {
                        12.0
                    };

                    painter.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{}", p.rect_id),
                        egui::FontId::proportional(font_size),
                        text_color,
                    );
                }
            }

            ui.add_space(6.0);
        });
    }
}

impl eframe::App for PackingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Rectangle Packing GUI");

                ui.horizontal(|ui| {
                    ui.label("Rectangles");
                    ui.add(
                        egui::DragValue::new(&mut self.num_rects)
                            .clamp_range(1..=1000)
                            .speed(1),
                    );
                });

                ui.add(egui::Slider::new(&mut self.min_size, 1..=10).text("Min size"));
                ui.add(egui::Slider::new(&mut self.max_size, 1..=10).text("Max size"));
                ui.add(egui::Slider::new(&mut self.box_size, 5..=20).text("Box size"));

                if ui.button("Generate Instance").clicked() {
                    self.instance = Some(generate_instance(
                        self.num_rects,
                        self.min_size,
                        self.max_size,
                        self.min_size,
                        self.max_size,
                        self.box_size,
                    ));
                    self.solution = None;
                    self.history.clear();
                    self.history_index = 0;
                    self.status = format!("Instance generated with {} rectangles", self.num_rects);
                }

                egui::ComboBox::from_label("Algorithm")
                    .selected_text(match self.selected_algo {
                        0 => "Greedy Area",
                        1 => "Greedy Max Side",
                        2 => "Local Search Permutation",
                        3 => "Local Search Geometry",
                        _ => "Local Search Overlap",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_algo, 0, "Greedy Area");
                        ui.selectable_value(&mut self.selected_algo, 1, "Greedy Max Side");
                        ui.selectable_value(&mut self.selected_algo, 2, "Local Search Permutation");
                        ui.selectable_value(&mut self.selected_algo, 3, "Local Search Geometry");
                        ui.selectable_value(&mut self.selected_algo, 4, "Local Search Overlap");
                    });

                if ui.button("Run Algorithm").clicked() {
                    if let Some(instance) = &self.instance {
                        match self.selected_algo {
                            0 => {
                                let sol = run_greedy(instance, GreedyStrategy::AreaDescending);
                                let score = evaluate(&sol, instance.box_size);
                                self.status = format!(
                                    "Greedy Area → bins={}, waste={}",
                                    score.bins_used, score.wasted_area
                                );
                                self.set_history_filtered(vec![sol]);
                            }
                            1 => {
                                let sol = run_greedy(instance, GreedyStrategy::MaxSideDescending);
                                let score = evaluate(&sol, instance.box_size);
                                self.status = format!(
                                    "Greedy Max → bins={}, waste={}",
                                    score.bins_used, score.wasted_area
                                );
                                self.set_history_filtered(vec![sol]);
                            }
                            2 => {
                                let result = run_local_search_swap(
                                    instance,
                                    50,
                                    LocalSearchStart::BadAscendingArea,
                                );

                                self.status = format!(
                                    "Permutation LS → bins={}, waste={}, iterations={}, snapshots={}",
                                    result.score.bins_used,
                                    result.score.wasted_area,
                                    result.iterations,
                                    result.history.len()
                                );

                                if instance.rectangles.len() <= 25 {
                                    self.set_history_raw(result.history);
                                } else {
                                    self.set_history_filtered(result.history);
                                }
                            }
                            3 => {
                                let start = if instance.rectangles.len() <= 25 {
                                    Self::build_scattered_solution(instance)
                                } else {
                                    run_greedy(instance, GreedyStrategy::AreaDescending)
                                };

                                let result = run_geometry_local_search(instance, start, 100);

                                self.status = format!(
                                    "Geometry LS → bins={}, waste={}, iterations={}, snapshots={}",
                                    result.score.bins_used,
                                    result.score.wasted_area,
                                    result.iterations,
                                    result.history.len()
                                );

                                if instance.rectangles.len() <= 25 {
                                    self.set_history_raw(result.history);
                                } else {
                                    self.set_history_filtered(result.history);
                                }
                            }
                            4 => {
                                if instance.rectangles.len() <= 25 {
                                    let start = Self::build_scattered_solution(instance);
                                    let result =
                                        run_overlap_local_search_with_start(instance, start, 100);

                                    self.status = format!(
                                        "Overlap LS → bins={}, waste={}, iterations={}, snapshots={}",
                                        result.score.bins_used,
                                        result.score.wasted_area,
                                        result.iterations,
                                        result.history.len()
                                    );

                                    self.set_history_raw(result.history);
                                } else {
                                    let result = run_overlap_local_search(instance, 100);

                                    self.status = format!(
                                        "Overlap LS → bins={}, waste={}, iterations={}, snapshots={}",
                                        result.score.bins_used,
                                        result.score.wasted_area,
                                        result.iterations,
                                        result.history.len()
                                    );

                                    self.set_history_filtered(result.history);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        self.status = "Generate an instance first.".to_string();
                    }
                }

                ui.separator();
                ui.label(&self.status);

                ui.separator();
                ui.heading("Tracking");

                ui.checkbox(
                    &mut self.auto_track_most_changed,
                    "Auto-track most changed rectangle",
                );

                ui.label("Manual rectangle IDs (comma-separated)");
                let response = ui.text_edit_singleline(&mut self.tracked_rectangles_input);
                if response.changed() {
                    self.parse_tracked_rectangles();
                }

                if ui.button("Clear Manual Tracking").clicked() {
                    self.tracked_rectangles_input.clear();
                    self.tracked_rectangles.clear();
                }

                let effective_tracked = self.effective_tracked_rectangles();
                if effective_tracked.is_empty() {
                    ui.label("No tracked rectangles.");
                } else {
                    ui.label(format!("Tracked: {:?}", effective_tracked));
                }

                if !self.history.is_empty() {
                    ui.separator();
                    ui.heading("Snapshots");

                    ui.horizontal(|ui| {
                        let prev_enabled = self.history_index > 0;
                        if ui
                            .add_enabled(prev_enabled, egui::Button::new("Previous"))
                            .clicked()
                        {
                            self.history_index -= 1;
                        }

                        let next_enabled = self.history_index + 1 < self.history.len();
                        if ui
                            .add_enabled(next_enabled, egui::Button::new("Next"))
                            .clicked()
                        {
                            self.history_index += 1;
                        }
                    });

                    ui.label(format!(
                        "Snapshot {}/{}",
                        self.history_index + 1,
                        self.history.len()
                    ));

                    if self.selected_algo >= 2 {
                        if let (Some(prev), Some(curr)) =
                            (self.previous_display_solution(), self.current_display_solution())
                        {
                            let changed = Self::changed_rectangles_between(prev, curr);
                            if changed.is_empty() {
                                ui.label("Step change: no visible rectangle change.");
                            } else {
                                ui.label(format!("Step change: rectangles moved {:?}", changed));
                            }
                        } else {
                            ui.label("Step change: initial state.");
                        }
                    }
                }

                if let Some(instance) = &self.instance {
                    ui.separator();

                    ui.collapsing("Current Instance", |ui| {
                        ui.label(format!(
                            "Rectangles: {}, Box size: {}",
                            instance.rectangles.len(),
                            instance.box_size
                        ));

                        ui.collapsing("Show rectangles", |ui| {
                            for rect in &instance.rectangles {
                                ui.label(format!(
                                    "Rect {}: {}x{}",
                                    rect.id, rect.width, rect.height
                                ));
                            }
                        });
                    });
                }

                if let Some(solution) = self.current_display_solution() {
                    ui.separator();

                    let score = evaluate(solution, self.box_size);

                    ui.collapsing("Performance", |ui| {
                        ui.label(format!("Bins used: {}", score.bins_used));
                        ui.label(format!("Wasted area: {}", score.wasted_area));
                    });

                    ui.collapsing("Tracked Rectangle Info", |ui| {
                        let tracked = self.effective_tracked_rectangles();

                        if tracked.is_empty() {
                            ui.label("No tracked rectangles.");
                        } else {
                            for rect_id in tracked {
                                ui.separator();
                                ui.label(format!("Rectangle {}", rect_id));

                                let current = Self::find_rect_position(solution, rect_id);
                                let previous = self
                                    .previous_display_solution()
                                    .and_then(|prev| Self::find_rect_position(prev, rect_id));

                                match previous {
                                    Some((bin, x, y, w, h, rot)) => {
                                        ui.label(format!(
                                            "Previous: Bin {}, pos=({}, {}), size={}x{}, rotated={}",
                                            bin, x, y, w, h, rot
                                        ));
                                    }
                                    None => {
                                        ui.label("Previous: not available");
                                    }
                                }

                                match current {
                                    Some((bin, x, y, w, h, rot)) => {
                                        ui.label(format!(
                                            "Current: Bin {}, pos=({}, {}), size={}x{}, rotated={}",
                                            bin, x, y, w, h, rot
                                        ));
                                    }
                                    None => {
                                        ui.label("Current: not found");
                                    }
                                }
                            }
                        }
                    });

                    ui.collapsing("Solution Summary", |ui| {
                        for (bin_index, bin) in solution.bins.iter().enumerate() {
                            ui.collapsing(format!("Bin {}", bin_index), |ui| {
                                for p in &bin.placements {
                                    ui.label(format!(
                                        "Rect {} at ({}, {}) size {}x{} rotated={}",
                                        p.rect_id, p.x, p.y, p.width, p.height, p.rotated
                                    ));
                                }
                            });
                        }
                    });
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if let Some(solution) = self.current_display_solution() {
                                let focus_ids = self.visual_focus_rectangles();
                                let bins_per_row = 8;

                                for row in solution.bins.chunks(bins_per_row) {
                                    ui.horizontal(|ui| {
                                        for (offset, bin) in row.iter().enumerate() {
                                            let bin_index = solution
                                                .bins
                                                .iter()
                                                .position(|b| b.id == bin.id)
                                                .unwrap_or(offset);

                                            self.draw_bin(ui, bin_index, bin, &focus_ids);
                                            ui.add_space(10.0);
                                        }
                                    });
                                    ui.add_space(12.0);
                                }
                            } else {
                                ui.label("Generate an instance and run an algorithm.");
                            }
                        });
                });
        });
    }
}