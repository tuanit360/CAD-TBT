//! Preliminary BIM collaboration workspace for CAD 360 Viet Nam - TBT.
//! This module intentionally owns only coordination metadata/UI. CAD geometry stays in the
//! canonical CadApp/Session and will be bound to these scopes incrementally.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Discipline {
    Architecture,
    Structure,
    Electrical,
    Plumbing,
    Hvac,
    Coordination,
}

impl Discipline {
    pub const ALL: [Self; 6] = [
        Self::Architecture,
        Self::Structure,
        Self::Electrical,
        Self::Plumbing,
        Self::Hvac,
        Self::Coordination,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Architecture => "Kiến trúc",
            Self::Structure => "Kết cấu",
            Self::Electrical => "Điện",
            Self::Plumbing => "Nước",
            Self::Hvac => "Điều hòa/Thông gió",
            Self::Coordination => "Phối hợp",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionMode {
    Offline,
    Online,
}

impl ConnectionMode {
    fn label(self) -> &'static str {
        match self {
            Self::Offline => "Offline cá nhân",
            Self::Online => "Online cộng tác",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewLayout {
    One,
    Two,
    Three,
    Four,
}

impl ViewLayout {
    fn slots(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::One => "1 cửa sổ",
            Self::Two => "2 cửa sổ",
            Self::Three => "3 cửa sổ",
            Self::Four => "4 cửa sổ",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub id: u64,
    pub name: String,
    /// BIM absolute/project elevation in metres.
    pub elevation_m: f64,
    pub height_m: f64,
}

impl Floor {
    pub fn world_z(&self, local_z_m: f64) -> f64 {
        self.elevation_m + local_z_m
    }
}

#[derive(Clone, Debug)]
pub struct UserAccess {
    pub account_id: String,
    pub display_name: String,
    pub discipline: Discipline,
    pub can_edit: bool,
    pub can_view_other_disciplines: bool,
    pub can_view_other_floors: bool,
}

#[derive(Clone, Debug)]
pub struct ViewSlot {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub owner_account_id: Option<String>,
    pub overlay_opacity: f32,
    pub show_other_users: bool,
}

#[derive(Clone, Debug)]
pub struct ProjectCoordinates {
    pub origin_x_m: f64,
    pub origin_y_m: f64,
    pub origin_z_m: f64,
    pub rotation_deg: f64,
}

impl Default for ProjectCoordinates {
    fn default() -> Self {
        Self { origin_x_m: 0.0, origin_y_m: 0.0, origin_z_m: 0.0, rotation_deg: 0.0 }
    }
}

pub struct State {
    pub connection: ConnectionMode,
    pub layout: ViewLayout,
    pub current_account_id: String,
    pub coordinates: ProjectCoordinates,
    pub floors: Vec<Floor>,
    pub users: Vec<UserAccess>,
    pub views: Vec<ViewSlot>,
    pub dirty_local_changes: usize,
}

impl Default for State {
    fn default() -> Self {
        let floors = vec![
            Floor { id: 1, name: "Tầng 1".into(), elevation_m: 0.0, height_m: 3.6 },
            Floor { id: 2, name: "Tầng 2".into(), elevation_m: 3.6, height_m: 3.6 },
            Floor { id: 3, name: "Tầng 3".into(), elevation_m: 7.2, height_m: 3.6 },
        ];
        let users = vec![
            UserAccess { account_id: "local-owner".into(), display_name: "Người dùng hiện tại".into(), discipline: Discipline::Architecture, can_edit: true, can_view_other_disciplines: true, can_view_other_floors: true },
            UserAccess { account_id: "struct-01".into(), display_name: "Kỹ sư kết cấu".into(), discipline: Discipline::Structure, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
            UserAccess { account_id: "mep-elec-01".into(), display_name: "Kỹ sư điện".into(), discipline: Discipline::Electrical, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
            UserAccess { account_id: "mep-water-01".into(), display_name: "Kỹ sư nước".into(), discipline: Discipline::Plumbing, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
        ];
        let views = vec![
            ViewSlot { floor_id: 1, discipline: Discipline::Architecture, owner_account_id: Some("local-owner".into()), overlay_opacity: 1.0, show_other_users: false },
            ViewSlot { floor_id: 1, discipline: Discipline::Structure, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true },
            ViewSlot { floor_id: 1, discipline: Discipline::Electrical, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true },
            ViewSlot { floor_id: 1, discipline: Discipline::Plumbing, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true },
        ];
        Self {
            connection: ConnectionMode::Offline,
            layout: ViewLayout::Two,
            current_account_id: "local-owner".into(),
            coordinates: ProjectCoordinates::default(),
            floors,
            users,
            views,
            dirty_local_changes: 0,
        }
    }
}

impl State {
    pub fn active_user(&self) -> Option<&UserAccess> {
        self.users.iter().find(|u| u.account_id == self.current_account_id)
    }

    pub fn can_view(&self, target: &UserAccess, target_floor: u64, active_floor: u64) -> bool {
        let Some(me) = self.active_user() else { return false };
        if target.account_id == me.account_id {
            return target_floor == active_floor || me.can_view_other_floors;
        }
        (me.can_view_other_disciplines || target.discipline == me.discipline)
            && (target_floor == active_floor || me.can_view_other_floors)
    }

    pub fn world_position(&self, floor_id: u64, local_x_m: f64, local_y_m: f64, local_z_m: f64) -> Option<[f64; 3]> {
        let floor = self.floors.iter().find(|f| f.id == floor_id)?;
        let angle = self.coordinates.rotation_deg.to_radians();
        let cos = angle.cos();
        let sin = angle.sin();
        let rotated_x = local_x_m * cos - local_y_m * sin;
        let rotated_y = local_x_m * sin + local_y_m * cos;
        Some([
            self.coordinates.origin_x_m + rotated_x,
            self.coordinates.origin_y_m + rotated_y,
            self.coordinates.origin_z_m + floor.world_z(local_z_m),
        ])
    }
}

pub fn panel(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("BIM Cộng tác");
    ui.label("Khung sơ bộ cho thiết kế nhiều bộ môn, nhiều tầng/cao độ và phối hợp online/offline trong cùng một file dự án.");

    ui.horizontal(|ui| {
        ui.label("Chế độ:");
        for mode in [ConnectionMode::Offline, ConnectionMode::Online] {
            ui.selectable_value(&mut state.connection, mode, mode.label());
        }
        ui.separator();
        egui::ComboBox::from_id_salt("bim_current_account")
            .selected_text(state.users.iter().find(|u| u.account_id == state.current_account_id).map(|u| u.display_name.as_str()).unwrap_or("Chưa đăng nhập"))
            .show_ui(ui, |ui| {
                for user in &state.users {
                    ui.selectable_value(&mut state.current_account_id, user.account_id.clone(), &user.display_name);
                }
            });
        if state.connection == ConnectionMode::Offline {
            ui.label(format!("Thay đổi cục bộ chưa đồng bộ: {}", state.dirty_local_changes));
        }
    });

    ui.separator();
    ui.collapsing("Tọa độ dự án / BIM", |ui| {
        egui::Grid::new("bim_coordinates").show(ui, |ui| {
            ui.label("Origin X (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_x_m)); ui.end_row();
            ui.label("Origin Y (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_y_m)); ui.end_row();
            ui.label("Origin Z (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_z_m)); ui.end_row();
            ui.label("Góc xoay dự án (°)"); ui.add(egui::DragValue::new(&mut state.coordinates.rotation_deg)); ui.end_row();
        });
        ui.label("Mọi đối tượng BIM sẽ được lưu theo project coordinates + tầng/cao độ + tọa độ local.");
    });

    ui.collapsing("Tầng / Cao độ", |ui| {
        egui::Grid::new("bim_floors").striped(true).show(ui, |ui| {
            ui.strong("Tên tầng"); ui.strong("Cao độ (m)"); ui.strong("Chiều cao tầng (m)"); ui.end_row();
            for floor in &mut state.floors {
                ui.text_edit_singleline(&mut floor.name);
                ui.add(egui::DragValue::new(&mut floor.elevation_m).speed(0.1));
                ui.add(egui::DragValue::new(&mut floor.height_m).speed(0.1).range(0.1..=100.0));
                ui.end_row();
            }
        });
        if ui.button("+ Thêm tầng").clicked() {
            let next_id = state.floors.iter().map(|f| f.id).max().unwrap_or(0) + 1;
            let elevation = state.floors.last().map(|f| f.elevation_m + f.height_m).unwrap_or(0.0);
            state.floors.push(Floor { id: next_id, name: format!("Tầng {}", next_id), elevation_m: elevation, height_m: 3.6 });
        }
    });

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Chia màn hình:");
        for layout in [ViewLayout::One, ViewLayout::Two, ViewLayout::Three, ViewLayout::Four] {
            ui.selectable_value(&mut state.layout, layout, layout.label());
        }
    });

    let slots = state.layout.slots();
    egui::Grid::new("bim_view_grid").num_columns(if slots == 1 { 1 } else { 2 }).spacing([8.0, 8.0]).show(ui, |ui| {
        for i in 0..slots {
            let slot = &mut state.views[i];
            ui.group(|ui| {
                ui.set_min_width(320.0);
                ui.strong(format!("Viewport {}", i + 1));
                egui::ComboBox::from_id_salt(("floor", i))
                    .selected_text(state.floors.iter().find(|f| f.id == slot.floor_id).map(|f| f.name.as_str()).unwrap_or("Chưa chọn tầng"))
                    .show_ui(ui, |ui| {
                        for floor in &state.floors {
                            ui.selectable_value(&mut slot.floor_id, floor.id, &floor.name);
                        }
                    });
                egui::ComboBox::from_id_salt(("discipline", i))
                    .selected_text(slot.discipline.label())
                    .show_ui(ui, |ui| {
                        for discipline in Discipline::ALL {
                            ui.selectable_value(&mut slot.discipline, discipline, discipline.label());
                        }
                    });
                ui.checkbox(&mut slot.show_other_users, "Hiển thị thiết kế của người khác");
                ui.add(egui::Slider::new(&mut slot.overlay_opacity, 0.05..=1.0).text("Độ đậm overlay"));
                ui.label(if i == 0 { "Viewport làm việc chính" } else { "Viewport phối hợp/xem trực tiếp" });
            });
            if slots > 1 && i % 2 == 1 { ui.end_row(); }
        }
    });

    ui.separator();
    ui.collapsing("Quyền tài khoản", |ui| {
        egui::Grid::new("bim_access").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Bộ môn"); ui.strong("Sửa"); ui.strong("Xem bộ môn khác"); ui.strong("Xem tầng khác"); ui.end_row();
            for user in &mut state.users {
                ui.label(&user.display_name);
                ui.label(user.discipline.label());
                ui.checkbox(&mut user.can_edit, "");
                ui.checkbox(&mut user.can_view_other_disciplines, "");
                ui.checkbox(&mut user.can_view_other_floors, "");
                ui.end_row();
            }
        });
    });

    ui.colored_label(
        egui::Color32::YELLOW,
        "Giai đoạn sơ bộ: viewport hiện mới quản lý scope/tầng/bộ môn/opacity. Renderer đa viewport, server sync và khóa đối tượng sẽ được nối vào các cấu trúc này ở bước tiếp theo."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_elevation_maps_local_to_world_z() {
        let state = State::default();
        assert_eq!(state.world_position(2, 10.0, 20.0, 0.5), Some([10.0, 20.0, 4.1]));
    }

    #[test]
    fn project_rotation_is_applied_to_world_coordinates() {
        let mut state = State::default();
        state.coordinates.rotation_deg = 90.0;
        let p = state.world_position(1, 10.0, 0.0, 0.0).unwrap();
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn view_layout_has_expected_slot_count() {
        assert_eq!(ViewLayout::One.slots(), 1);
        assert_eq!(ViewLayout::Two.slots(), 2);
        assert_eq!(ViewLayout::Three.slots(), 3);
        assert_eq!(ViewLayout::Four.slots(), 4);
    }

    #[test]
    fn owner_can_view_other_disciplines_and_floors_when_granted() {
        let state = State::default();
        let target = state.users.iter().find(|u| u.account_id == "struct-01").unwrap();
        assert!(state.can_view(target, 2, 1));
    }

    #[test]
    fn restricted_user_cannot_cross_discipline_without_grant() {
        let mut state = State::default();
        state.current_account_id = "struct-01".into();
        let target = state.users.iter().find(|u| u.account_id == "mep-elec-01").unwrap();
        assert!(!state.can_view(target, 1, 1));
    }
}
