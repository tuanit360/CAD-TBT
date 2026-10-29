//! Live, read-only quantities from the canonical CAD drawing.
//! Geometry changes are owned by the native CAD command/undo system.
//! These are drawing-unit quantities, NOT validated BIM/IFC/steel QTO.
use std::collections::BTreeMap;

use cadcraft_doc::EntityKind;
use cadcraft_ui_egui::CadApp;

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
    if let Some(drawing) = app.session.doc() {
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

pub fn show(ctx: &egui::Context, open: &mut bool, app: &CadApp) {
    let groups = quantities(app);
    egui::Window::new("Đo bóc / Takeoff — CAD native")
        .open(open)
        .default_width(750.0)
        .show(ctx, |ui| {
            ui.label("Số liệu trực tiếp từ bản vẽ CAD đang mở — thay đổi sau khi vẽ/sửa/Undo/Redo.");
            ui.colored_label(egui::Color32::YELLOW,
                "Đơn vị theo bản vẽ; không phải m/m² nếu drawing chưa thiết lập đúng. Không tính Solid/IFC/PDF, không tự suy diễn khối lượng thép.");
            ui.separator();
            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui::Grid::new("native_qto_table").striped(true).show(ui, |ui| {
                    for heading in ["Layer", "Số đối tượng", "LINE dài", "ARC dài", "PLINE dài", "PLINE kín diện tích", "CIRCLE diện tích", "Khác/chưa đo"] {
                        ui.strong(heading);
                    }
                    ui.end_row();
                    for (layer, q) in &groups {
                        ui.label(layer);
                        ui.label(q.count.to_string());
                        ui.label(format!("{:.3}", q.line_length));
                        ui.label(format!("{:.3}", q.arc_length));
                        ui.label(format!("{:.3}", q.polyline_length));
                        ui.label(format!("{:.3}", q.closed_area));
                        ui.label(format!("{:.3}", q.circle_area));
                        ui.label(q.unsupported.to_string());
                        ui.end_row();
                    }
                });
            });
            if ui.button("Sao chép bảng CSV / Copy CSV").clicked() {
                let mut csv = String::from("layer,entity_count,line_length_units,arc_length_units,polyline_length_units,closed_polyline_area_units2,circle_area_units2,unmeasured_count\n");
                for (layer, q) in &groups {
                    let safe = format!("\"{}\"", layer.replace('"', "\"\""));
                    csv.push_str(&format!("{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{}\n",
                        safe, q.count, q.line_length, q.arc_length, q.polyline_length, q.closed_area, q.circle_area, q.unsupported));
                }
                ctx.copy_text(csv);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::geom::PolyVertex;
    use cadcraft_doc::geom::Vec2;

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
