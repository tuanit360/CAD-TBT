//! Live, read-only quantities from the canonical CAD drawing.
//! Geometry changes are owned by the native CAD command/undo system.
//! These are drawing-unit quantities, NOT validated BIM/IFC/steel QTO.
use std::collections::BTreeMap;

use cadcraft_doc::EntityKind;
use cadcraft_ui_egui::CadApp;

#[derive(Default)]
pub struct State {
    drawing_distance: f64,
    real_distance_m: f64,
}

impl State {
    fn scale_to_m(&self) -> Option<f64> {
        if self.drawing_distance.is_finite() && self.real_distance_m.is_finite() && self.drawing_distance > 1e-12 && self.real_distance_m > 0.0 {
            Some(self.real_distance_m / self.drawing_distance)
        } else {
            None
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": 1,
            "drawing_distance": self.drawing_distance,
            "real_distance_m": self.real_distance_m
        })
    }

    fn load_json(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let drawing_distance = value.get("drawing_distance").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu drawing_distance".to_string())?;
        let real_distance_m = value.get("real_distance_m").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu real_distance_m".to_string())?;
        let candidate = Self { drawing_distance, real_distance_m };
        if candidate.scale_to_m().is_none() { return Err("Dữ liệu hiệu chỉnh tỷ lệ không hợp lệ".into()); }
        *self = candidate;
        Ok(())
    }
}

#[derive(Default)]
struct Quantity {
    count: usize,
    line_length: f64,
    arc_length: f64,
    polyline_length: f64,
    closed_area: f64,
    circle_area: f64,
    unsupported: usize,
}

fn bulge_edge(a: (f64, f64), b: (f64, f64), bulge: f64) -> Option<(f64, f64)> {
    if ![a.0, a.1, b.0, b.1].iter().all(|value| value.is_finite()) {
        return None;
    }
    let chord = (b.0 - a.0).hypot(b.1 - a.1);
    if !chord.is_finite() || !bulge.is_finite() {
        return None;
    }
    if chord <= 1e-12 && bulge.abs() >= 1e-12 { return None; }
    if bulge.abs() < 1e-12 {
        return Some((chord, 0.0));
    }
    let theta = 4.0 * bulge.atan();
    let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
    let length = radius * theta.abs();
    // Signed circular segment area, retained when calculating closed loops.
    let segment = 0.5 * radius * radius * (theta - theta.sin());
    if length.is_finite() && segment.is_finite() {
        Some((length, segment))
    } else {
        None
    }
}

fn segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }
    fn on_segment(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        let eps = 1e-10;
        c.0 >= a.0.min(b.0) - eps && c.0 <= a.0.max(b.0) + eps
            && c.1 >= a.1.min(b.1) - eps && c.1 <= a.1.max(b.1) + eps
    }
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let eps = 1e-10;
    if o1.abs() <= eps && on_segment(a, b, c) { return true; }
    if o2.abs() <= eps && on_segment(a, b, d) { return true; }
    if o3.abs() <= eps && on_segment(c, d, a) { return true; }
    if o4.abs() <= eps && on_segment(c, d, b) { return true; }
    (o1 > eps && o2 < -eps || o1 < -eps && o2 > eps)
        && (o3 > eps && o4 < -eps || o3 < -eps && o4 > eps)
}

fn lwpoly_metrics(vertices: &[cadcraft_doc::geom::PolyVertex], closed: bool) -> Option<(f64, f64)> {
    if vertices.iter().any(|vertex| !vertex.p.x.is_finite() || !vertex.p.y.is_finite() || !vertex.bulge.is_finite()) {
        return None;
    }
    if vertices.len() < 2 {
        return Some((0.0, 0.0));
    }
    if closed {
        if vertices.len() < 3 { return None; }
        // Conservative: reject loops with crossing straight chords. Bulged arcs
        // are still measured, but more complex arc/arc crossings require CAD validation.
        for i in 0..vertices.len() {
            let a = (vertices[i].p.x, vertices[i].p.y);
            let b = (vertices[(i + 1) % vertices.len()].p.x,
                     vertices[(i + 1) % vertices.len()].p.y);
            if (b.0 - a.0).hypot(b.1 - a.1) < 1e-10 { return None; }
            for j in (i + 1)..vertices.len() {
                if j == i + 1 || (i == 0 && j == vertices.len() - 1) { continue; }
                let c = (vertices[j].p.x, vertices[j].p.y);
                let d = (vertices[(j + 1) % vertices.len()].p.x,
                         vertices[(j + 1) % vertices.len()].p.y);
                if segments_intersect(a, b, c, d) { return None; }
            }
        }
    }
    let edges = if closed { vertices.len() } else { vertices.len() - 1 };
    let mut length = 0.0;
    let mut signed_area_twice = 0.0;
    let mut segments_area = 0.0;
    for i in 0..edges {
        let a = &vertices[i];
        let b = &vertices[(i + 1) % vertices.len()];
        let (edge_length, segment) = bulge_edge((a.p.x, a.p.y), (b.p.x, b.p.y), a.bulge)?;
        length += edge_length;
        if closed {
            signed_area_twice += a.p.x * b.p.y - b.p.x * a.p.y;
            segments_area += segment;
        }
    }
    let area = if closed { (signed_area_twice * 0.5 + segments_area).abs() } else { 0.0 };
    if length.is_finite() && area.is_finite() {
        Some((length, area))
    } else {
        None
    }
}

fn quantities(app: &CadApp) -> BTreeMap<String, Quantity> {
    let mut groups = BTreeMap::<String, Quantity>::new();
    if let Ok(drawing) = app.session.doc() {
        for entity in drawing.model.iter() {
            let group = groups.entry(entity.common.layer.clone()).or_default();
            group.count += 1;
            match &entity.kind {
                EntityKind::Line(line) => {
                    let distance = (line.b.x - line.a.x).hypot(line.b.y - line.a.y)
                        .hypot(line.b.z - line.a.z);
                    if distance.is_finite() { group.line_length += distance; } else { group.unsupported += 1; }
                }
                EntityKind::Circle(circle) => {
                    if circle.radius.is_finite() && circle.radius > 0.0 {
                        group.circle_area += std::f64::consts::PI * circle.radius.powi(2);
                    } else { group.unsupported += 1; }
                }
                EntityKind::Arc(arc) => {
                    if arc.radius.is_finite() && arc.radius > 0.0 && arc.start.is_finite() && arc.end.is_finite() {
                        let delta = (arc.end - arc.start).rem_euclid(std::f64::consts::TAU);
                        group.arc_length += arc.radius * delta;
                    } else { group.unsupported += 1; }
                }
                EntityKind::LwPolyline(poly) => {
                    match lwpoly_metrics(&poly.vertices, poly.closed) {
                        Some((length, area)) => { group.polyline_length += length; group.closed_area += area; }
                        None => group.unsupported += 1,
                    }
                }
                EntityKind::Polyline3d(poly) => {
                    let n = poly.points.len();
                    let edges = if poly.closed { n } else { n.saturating_sub(1) };
                    let mut length = 0.0;
                    for i in 0..edges {
                        let a = &poly.points[i];
                        let b = &poly.points[(i + 1) % n];
                        length += (b.x - a.x).hypot(b.y - a.y).hypot(b.z - a.z);
                    }
                    if n >= 2 && poly.points.iter().all(|p|
                        p.x.is_finite() && p.y.is_finite() && p.z.is_finite())
                        && length.is_finite() {
                        group.polyline_length += length;
                    } else { group.unsupported += 1; }
                }
                _ => group.unsupported += 1,
            }
        }
    }
    groups
}

fn draw_contents(ui: &mut egui::Ui, app: &CadApp, state: &mut State) {
    let groups = quantities(app);
    ui.heading("Đo bóc khối lượng — CAD native");
    ui.label("Số liệu trực tiếp từ bản vẽ CAD đang mở; thay đổi ngay sau khi vẽ, sửa, Undo hoặc Redo.");
    ui.separator();

    ui.group(|ui| {
        ui.strong("Hiệu chỉnh tỷ lệ");
        ui.horizontal(|ui| {
            ui.label("Khoảng cách trên bản vẽ:");
            ui.add(egui::DragValue::new(&mut state.drawing_distance).speed(0.1).range(0.0..=f64::MAX));
            ui.label("Chiều dài thực (m):");
            ui.add(egui::DragValue::new(&mut state.real_distance_m).speed(0.1).range(0.0..=f64::MAX));
        });
        match state.scale_to_m() {
            Some(scale) => {
                ui.label(format!("Tỷ lệ quy đổi: 1 đơn vị bản vẽ = {:.8} m", scale));
            }
            None => {
                ui.colored_label(egui::Color32::YELLOW, "Chưa hiệu chỉnh tỷ lệ. Kết quả đang hiển thị theo đơn vị bản vẽ.");
            }
        }
    });

    let scale = state.scale_to_m();
    let length_unit = if scale.is_some() { "m" } else { "đv" };
    let area_unit = if scale.is_some() { "m²" } else { "đv²" };
    let lf = scale.unwrap_or(1.0);
    let af = lf * lf;

    ui.horizontal(|ui| {
        if ui.button("Lưu dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).set_file_name("TBT360_QTO_PROJECT.json").save_file() {
                match serde_json::to_vec_pretty(&state.to_json()).map_err(|e| e.to_string()).and_then(|data| std::fs::write(&path, data).map_err(|e| e.to_string())) {
                    Ok(()) => ui.ctx().debug_painter().text(egui::pos2(16.0, 16.0), egui::Align2::LEFT_TOP, "Đã lưu dự án đo bóc", egui::FontId::proportional(14.0), egui::Color32::LIGHT_GREEN),
                    Err(e) => eprintln!("Không thể lưu dự án đo bóc: {e}"),
                };
            }
        }
        if ui.button("Mở dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).pick_file() {
                match std::fs::read(&path).map_err(|e| e.to_string())
                    .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).map_err(|e| e.to_string()))
                    .and_then(|value| state.load_json(&value)) {
                    Ok(()) => ui.ctx().debug_painter().text(egui::pos2(16.0, 34.0), egui::Align2::LEFT_TOP, "Đã mở dự án đo bóc", egui::FontId::proportional(14.0), egui::Color32::LIGHT_GREEN),
                    Err(e) => eprintln!("Không thể mở dự án đo bóc: {e}"),
                };
            }
        }
    });

    ui.separator();
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new("native_qto_table").striped(true).min_col_width(105.0).show(ui, |ui| {
            for heading in ["Layer", "Số đối tượng", "LINE", "ARC", "PLINE", "PLINE kín", "CIRCLE", "Khác/chưa đo"] {
                ui.strong(heading);
            }
            ui.end_row();
            for (layer, q) in &groups {
                ui.label(layer);
                ui.label(q.count.to_string());
                ui.label(format!("{:.3} {}", q.line_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.arc_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.polyline_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.closed_area * af, area_unit));
                ui.label(format!("{:.3} {}", q.circle_area * af, area_unit));
                ui.label(q.unsupported.to_string());
                ui.end_row();
            }
        });
    });

    ui.separator();
    if ui.button("Sao chép bảng CSV").clicked() {
        let mut csv = String::from("layer,entity_count,line_length,arc_length,polyline_length,closed_polyline_area,circle_area,unmeasured_count,length_unit,area_unit\n");
        for (layer, q) in &groups {
            let safe = format!("\"{}\"", layer.replace('"', "\"\""));
            csv.push_str(&format!("{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{}\n",
                safe, q.count, q.line_length * lf, q.arc_length * lf, q.polyline_length * lf, q.closed_area * af, q.circle_area * af, q.unsupported, length_unit, area_unit));
        }
        ui.ctx().copy_text(csv);
    }
}

pub fn panel(ui: &mut egui::Ui, app: &CadApp, state: &mut State) {
    draw_contents(ui, app, state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::geom::PolyVertex;
    use cadcraft_doc::geom::Vec2;

    #[test]
    fn qto_project_json_roundtrip_preserves_calibration() {
        let original = State { drawing_distance: 2500.0, real_distance_m: 2.5 };
        let mut restored = State::default();
        restored.load_json(&original.to_json()).unwrap();
        assert!((restored.drawing_distance - 2500.0).abs() < 1e-12);
        assert!((restored.real_distance_m - 2.5).abs() < 1e-12);
        assert!((restored.scale_to_m().unwrap() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn calibration_scale_converts_length_and_area() {
        let state = State { drawing_distance: 1000.0, real_distance_m: 1.0 };
        let scale = state.scale_to_m().unwrap();
        assert!((scale - 0.001).abs() < 1e-12);
        assert!((2500.0 * scale - 2.5).abs() < 1e-12);
        assert!((1_000_000.0 * scale * scale - 1.0).abs() < 1e-12);
    }

    #[test]
    fn invalid_calibration_is_rejected() {
        assert!(State::default().scale_to_m().is_none());
        assert!(State { drawing_distance: 0.0, real_distance_m: 1.0 }.scale_to_m().is_none());
        assert!(State { drawing_distance: 1.0, real_distance_m: 0.0 }.scale_to_m().is_none());
    }

    #[test]
    fn open_polyline_has_no_area() {
        let vertices = vec![PolyVertex::new(Vec2::new(0.0, 0.0)), PolyVertex::new(Vec2::new(3.0, 4.0))];
        assert_eq!(lwpoly_metrics(&vertices, false), Some((5.0, 0.0)));
    }

    #[test]
    fn closed_rectangle_area_and_perimeter() {
        let vertices = [(0.0,0.0),(10.0,0.0),(10.0,8.0),(0.0,8.0)]
            .iter().map(|&(x,y)| PolyVertex::new(Vec2::new(x,y))).collect::<Vec<_>>();
        let (perimeter, area) = lwpoly_metrics(&vertices, true).unwrap();
        assert!((perimeter - 36.0).abs() < 1e-10);
        assert!((area - 80.0).abs() < 1e-10);
    }
    #[test]
    fn semicircle_bulge_preserves_arc_and_segment_area() {
        let (length, area) = bulge_edge((0.0, 0.0), (2.0, 0.0), 1.0).unwrap();
        assert!((length - std::f64::consts::PI).abs() < 1e-10);
        assert!((area - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
    #[test]
    fn self_crossing_polyline_must_not_create_qto_area() {
        let points = [(0.0, 0.0), (4.0, 4.0), (0.0, 4.0), (4.0, 0.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn zero_length_closed_edge_is_invalid() {
        let points = [(0.0, 0.0), (0.0, 0.0), (2.0, 1.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn nonzero_bulge_with_coincident_endpoints_is_rejected() {
        assert!(bulge_edge((2.0, 3.0), (2.0, 3.0), 1.0).is_none());
        let mut vertices = vec![PolyVertex::new(Vec2::new(2.0, 3.0)), PolyVertex::new(Vec2::new(2.0, 3.0))];
        vertices[0].bulge = 1.0;
        assert!(lwpoly_metrics(&vertices, false).is_none());
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        assert!(bulge_edge((0.0, 0.0), (f64::NAN, 0.0), 0.0).is_none());
        assert!(bulge_edge((f64::NAN, 0.0), (1.0, 0.0), 0.0).is_none());
        let invalid = vec![
            PolyVertex::new(Vec2::new(f64::NAN, 0.0)),
            PolyVertex::new(Vec2::new(1.0, 0.0)),
        ];
        assert!(lwpoly_metrics(&invalid, true).is_none());
    }
}
