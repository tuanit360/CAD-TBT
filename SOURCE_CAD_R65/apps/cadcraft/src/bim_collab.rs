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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthStatus {
    SignedOut,
    OfflineProfile,
    OnlineAuthenticated,
}

#[derive(Clone, Debug)]
pub struct AuthSession {
    pub status: AuthStatus,
    pub account_id: String,
    pub display_name: String,
    pub email: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeStatus {
    NotStarted,
    InProgress,
    Review,
    Complete,
}

impl ScopeStatus {
    fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Chưa bắt đầu",
            Self::InProgress => "Đang thực hiện",
            Self::Review => "Đang kiểm tra",
            Self::Complete => "Hoàn thành",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScopeProgress {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub percent: u8,
    pub status: ScopeStatus,
    pub updated_by: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    Resolved,
}

impl IssueStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Open => "Mở",
            Self::InProgress => "Đang xử lý",
            Self::Resolved => "Đã xử lý",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CoordinationIssue {
    pub id: u64,
    pub floor_id: u64,
    pub source_discipline: Discipline,
    pub target_discipline: Discipline,
    pub handle: Option<cadcraft_doc::Handle>,
    pub assigned_to: String,
    pub description: String,
    pub status: IssueStatus,
}

#[derive(Clone, Debug)]
pub struct AccessRule {
    pub account_id: String,
    pub floor_id: Option<u64>,
    pub discipline: Option<Discipline>,
    pub can_view: bool,
    pub can_edit: bool,
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

const XDATA_APP: &str = "TBT360_BIM";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BimEntityMeta {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub owner_account_id: String,
    pub revision: u64,
}

impl BimEntityMeta {
    fn discipline_code(&self) -> &'static str {
        match self.discipline {
            Discipline::Architecture => "ARCH",
            Discipline::Structure => "STRUCT",
            Discipline::Electrical => "ELEC",
            Discipline::Plumbing => "PLUMB",
            Discipline::Hvac => "HVAC",
            Discipline::Coordination => "COORD",
        }
    }

    fn from_discipline_code(code: &str) -> Option<Discipline> {
        match code {
            "ARCH" => Some(Discipline::Architecture),
            "STRUCT" => Some(Discipline::Structure),
            "ELEC" => Some(Discipline::Electrical),
            "PLUMB" => Some(Discipline::Plumbing),
            "HVAC" => Some(Discipline::Hvac),
            "COORD" => Some(Discipline::Coordination),
            _ => None,
        }
    }

    fn payload(&self) -> String {
        format!("floor={};discipline={};owner={};revision={}", self.floor_id, self.discipline_code(), self.owner_account_id.replace(';', "_"), self.revision)
    }

    fn parse(payload: &str) -> Option<Self> {
        let mut floor_id = None;
        let mut discipline = None;
        let mut owner_account_id = None;
        let mut revision = None;
        for part in payload.split(';') {
            let (key, value) = part.split_once('=')?;
            match key {
                "floor" => floor_id = value.parse().ok(),
                "discipline" => discipline = Self::from_discipline_code(value),
                "owner" => owner_account_id = Some(value.to_string()),
                "revision" => revision = value.parse().ok(),
                _ => {}
            }
        }
        Some(Self { floor_id: floor_id?, discipline: discipline?, owner_account_id: owner_account_id?, revision: revision? })
    }
}

fn set_bim_meta(common: &mut cadcraft_doc::Common, meta: &BimEntityMeta) {
    let mut out = Vec::with_capacity(common.xdata.len() + 2);
    let mut skip_payload = false;
    for tag in &common.xdata {
        if tag.code == 1001 && tag.value == XDATA_APP {
            skip_payload = true;
            continue;
        }
        if skip_payload && tag.code >= 1000 && tag.code <= 1071 {
            skip_payload = false;
            continue;
        }
        skip_payload = false;
        out.push(tag.clone());
    }
    out.push(cadcraft_doc::RawTag { code: 1001, value: XDATA_APP.into() });
    out.push(cadcraft_doc::RawTag { code: 1000, value: meta.payload() });
    common.xdata = out;
}

fn bim_meta(common: &cadcraft_doc::Common) -> Option<BimEntityMeta> {
    common.xdata.windows(2).find_map(|pair| {
        if pair[0].code == 1001 && pair[0].value == XDATA_APP && pair[1].code == 1000 {
            BimEntityMeta::parse(&pair[1].value)
        } else {
            None
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
pub struct ChangeItem {
    pub handle: cadcraft_doc::Handle,
    pub operation: ChangeOperation,
    pub base_revision: u64,
    pub local_revision: u64,
}

#[derive(Clone, Debug)]
pub struct ChangeSet {
    pub id: u64,
    pub account_id: String,
    pub items: Vec<ChangeItem>,
    pub synced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncConflict {
    pub handle: cadcraft_doc::Handle,
    pub base_revision: u64,
    pub local_revision: u64,
    pub remote_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Presence {
    pub account_id: String,
    pub floor_id: u64,
    pub discipline: Discipline,
    pub selected_handle: Option<cadcraft_doc::Handle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLease {
    pub handle: cadcraft_doc::Handle,
    pub account_id: String,
    pub revision: u64,
}


fn detect_conflict(item: &ChangeItem, remote_revision: u64) -> Option<SyncConflict> {
    if remote_revision == item.base_revision {
        None
    } else {
        Some(SyncConflict {
            handle: item.handle,
            base_revision: item.base_revision,
            local_revision: item.local_revision,
            remote_revision,
        })
    }
}

fn slot_matches(meta: &BimEntityMeta, slot: &ViewSlot) -> bool {
    if meta.floor_id != slot.floor_id || meta.discipline != slot.discipline {
        return false;
    }
    match &slot.owner_account_id {
        Some(owner) => &meta.owner_account_id == owner,
        None => true,
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
    pub active_floor_id: u64,
    pub active_discipline: Discipline,
    pub pending_changes: Vec<ChangeSet>,
    next_change_set_id: u64,
    pub presence: Vec<Presence>,
    pub leases: Vec<ObjectLease>,
    pub conflicts: Vec<SyncConflict>,
    pub auth: AuthSession,
    pub access_rules: Vec<AccessRule>,
    pub progress: Vec<ScopeProgress>,
    pub issues: Vec<CoordinationIssue>,
    next_issue_id: u64,
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
            active_floor_id: 1,
            active_discipline: Discipline::Architecture,
            pending_changes: Vec::new(),
            next_change_set_id: 1,
            presence: Vec::new(),
            leases: Vec::new(),
            conflicts: Vec::new(),
            auth: AuthSession {
                status: AuthStatus::OfflineProfile,
                account_id: "local-owner".into(),
                display_name: "Người dùng hiện tại".into(),
                email: String::new(),
            },
            access_rules: vec![
                AccessRule { account_id: "local-owner".into(), floor_id: None, discipline: None, can_view: true, can_edit: true },
                AccessRule { account_id: "struct-01".into(), floor_id: None, discipline: Some(Discipline::Structure), can_view: true, can_edit: true },
                AccessRule { account_id: "mep-elec-01".into(), floor_id: None, discipline: Some(Discipline::Electrical), can_view: true, can_edit: true },
                AccessRule { account_id: "mep-water-01".into(), floor_id: None, discipline: Some(Discipline::Plumbing), can_view: true, can_edit: true },
            ],
            progress: vec![
                ScopeProgress { floor_id: 1, discipline: Discipline::Architecture, percent: 20, status: ScopeStatus::InProgress, updated_by: "local-owner".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Structure, percent: 10, status: ScopeStatus::InProgress, updated_by: "struct-01".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Electrical, percent: 0, status: ScopeStatus::NotStarted, updated_by: "mep-elec-01".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Plumbing, percent: 0, status: ScopeStatus::NotStarted, updated_by: "mep-water-01".into() },
            ],
            issues: Vec::new(),
            next_issue_id: 1,
        }
    }
}

impl State {
    fn ensure_progress_scope(&mut self, floor_id: u64, discipline: Discipline) {
        if self.progress.iter().any(|p| p.floor_id == floor_id && p.discipline == discipline) {
            return;
        }
        self.progress.push(ScopeProgress {
            floor_id,
            discipline,
            percent: 0,
            status: ScopeStatus::NotStarted,
            updated_by: self.current_account_id.clone(),
        });
    }

    fn create_issue_from_selection(&mut self, handle: Option<cadcraft_doc::Handle>, target_discipline: Discipline) {
        self.issues.push(CoordinationIssue {
            id: self.next_issue_id,
            floor_id: self.active_floor_id,
            source_discipline: self.active_discipline,
            target_discipline,
            handle,
            assigned_to: self.users.iter().find(|u| u.discipline == target_discipline).map(|u| u.account_id.clone()).unwrap_or_default(),
            description: "Kiểm tra phối hợp tại vị trí/đối tượng được chọn".into(),
            status: IssueStatus::Open,
        });
        self.next_issue_id += 1;
    }

    fn queue_update(&mut self, handle: cadcraft_doc::Handle, base_revision: u64, local_revision: u64) {
        let change = ChangeSet {
            id: self.next_change_set_id,
            account_id: self.current_account_id.clone(),
            items: vec![ChangeItem { handle, operation: ChangeOperation::Update, base_revision, local_revision }],
            synced: false,
        };
        self.next_change_set_id += 1;
        self.pending_changes.push(change);
        self.dirty_local_changes = self.pending_changes.iter().filter(|c| !c.synced).map(|c| c.items.len()).sum();
    }

    fn mark_all_synced(&mut self) {
        for change in &mut self.pending_changes {
            change.synced = true;
        }
        self.dirty_local_changes = 0;
    }

    fn login_offline(&mut self, account_id: String, display_name: String) {
        self.auth.status = AuthStatus::OfflineProfile;
        self.auth.account_id = account_id.clone();
        self.auth.display_name = display_name;
        self.auth.email.clear();
        self.current_account_id = account_id;
        self.connection = ConnectionMode::Offline;
    }

    fn login_online_stub(&mut self, account_id: String, display_name: String, email: String) {
        self.auth.status = AuthStatus::OnlineAuthenticated;
        self.auth.account_id = account_id.clone();
        self.auth.display_name = display_name;
        self.auth.email = email;
        self.current_account_id = account_id;
        self.connection = ConnectionMode::Online;
    }

    fn sign_out(&mut self) {
        self.auth.status = AuthStatus::SignedOut;
        self.auth.account_id.clear();
        self.auth.display_name.clear();
        self.auth.email.clear();
    }

    fn has_scope_access(&self, account_id: &str, floor_id: u64, discipline: Discipline, edit: bool) -> bool {
        self.access_rules.iter().any(|rule| {
            rule.account_id == account_id
                && rule.floor_id.is_none_or(|f| f == floor_id)
                && rule.discipline.is_none_or(|d| d == discipline)
                && if edit { rule.can_edit } else { rule.can_view }
        })
    }

    fn acquire_lease(&mut self, handle: cadcraft_doc::Handle, revision: u64) -> Result<(), String> {
        if let Some(existing) = self.leases.iter().find(|l| l.handle == handle) {
            if existing.account_id != self.current_account_id {
                return Err(format!("Đối tượng đang được {} chỉnh sửa", existing.account_id));
            }
            return Ok(());
        }
        self.leases.push(ObjectLease { handle, account_id: self.current_account_id.clone(), revision });
        Ok(())
    }

    fn release_lease(&mut self, handle: cadcraft_doc::Handle) {
        self.leases.retain(|l| !(l.handle == handle && l.account_id == self.current_account_id));
    }

    fn update_presence(&mut self, selected: Option<cadcraft_doc::Handle>) {
        let floor_id = self.active_floor_id;
        let discipline = self.active_discipline;
        if let Some(p) = self.presence.iter_mut().find(|p| p.account_id == self.current_account_id) {
            p.floor_id = floor_id;
            p.discipline = discipline;
            p.selected_handle = selected;
        } else {
            self.presence.push(Presence {
                account_id: self.current_account_id.clone(),
                floor_id,
                discipline,
                selected_handle: selected,
            });
        }
    }

    pub fn active_user(&self) -> Option<&UserAccess> {
        self.users.iter().find(|u| u.account_id == self.current_account_id)
    }

    pub fn can_view(&self, target: &UserAccess, target_floor: u64, active_floor: u64) -> bool {
        let Some(me) = self.active_user() else { return false };
        if target.account_id == me.account_id {
            return target_floor == active_floor || me.can_view_other_floors;
        }
        let broad = (me.can_view_other_disciplines || target.discipline == me.discipline)
            && (target_floor == active_floor || me.can_view_other_floors);
        broad && self.has_scope_access(&me.account_id, target_floor, target.discipline, false)
    }

    fn viewport_entity_count(&self, app: &cadcraft_ui_egui::CadApp, slot: &ViewSlot) -> usize {
        let Ok(doc) = app.session.doc() else { return 0 };
        doc.model.iter().filter(|entity| {
            let Some(meta) = bim_meta(&entity.common) else { return false };
            if !slot_matches(&meta, slot) { return false; }
            let Some(target) = self.users.iter().find(|u| u.account_id == meta.owner_account_id) else { return false };
            if !slot.show_other_users && meta.owner_account_id != self.current_account_id { return false; }
            self.can_view(target, meta.floor_id, self.active_floor_id)
        }).count()
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

pub fn panel(ui: &mut egui::Ui, app: &mut cadcraft_ui_egui::CadApp, state: &mut State) {
    ui.heading("BIM Cộng tác");
    ui.label("Khung sơ bộ cho thiết kế nhiều bộ môn, nhiều tầng/cao độ và phối hợp online/offline trong cùng một file dự án.");

    ui.group(|ui| {
        ui.strong("Đăng nhập / Phiên làm việc");
        match state.auth.status {
            AuthStatus::SignedOut => {
                ui.label("Chưa đăng nhập. Online sẽ nối TBT.Auth/OpenIddict; không lưu mật khẩu trong file CAD.");
            }
            AuthStatus::OfflineProfile => {
                ui.label(format!("Offline: {} ({})", state.auth.display_name, state.auth.account_id));
            }
            AuthStatus::OnlineAuthenticated => {
                ui.label(format!("Online: {} — {}", state.auth.display_name, state.auth.email));
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Dùng profile offline").clicked() {
                state.login_offline("local-owner".into(), "Người dùng hiện tại".into());
            }
            if ui.button("Mô phỏng đăng nhập TBT.Auth").clicked() {
                state.login_online_stub("local-owner".into(), "Người dùng hiện tại".into(), "user@tbt.local".into());
            }
            if ui.button("Đăng xuất").clicked() {
                state.sign_out();
            }
        });
    });

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
    ui.group(|ui| {
        ui.strong("Scope thiết kế hiện hành");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("bim_active_floor")
                .selected_text(state.floors.iter().find(|f| f.id == state.active_floor_id).map(|f| f.name.as_str()).unwrap_or("Chưa chọn tầng"))
                .show_ui(ui, |ui| {
                    for floor in &state.floors {
                        ui.selectable_value(&mut state.active_floor_id, floor.id, &floor.name);
                    }
                });
            egui::ComboBox::from_id_salt("bim_active_discipline")
                .selected_text(state.active_discipline.label())
                .show_ui(ui, |ui| {
                    for discipline in Discipline::ALL {
                        ui.selectable_value(&mut state.active_discipline, discipline, discipline.label());
                    }
                });
        });

        let selected = app.session.selection().to_vec();
        state.update_presence(selected.first().copied());
        ui.label(format!("Đối tượng đang chọn: {}", selected.len()));
        if ui.button("Gán tầng / bộ môn / chủ sở hữu cho đối tượng đang chọn").clicked() && !selected.is_empty() {
            let owner = state.current_account_id.clone();
            let floor_id = state.active_floor_id;
            let discipline = state.active_discipline;
            if let Ok(doc) = app.session.doc_mut() {
                for handle in selected {
                    let mut previous_revision = 0;
                    if let Some(entity) = doc.model.get(handle) {
                        previous_revision = bim_meta(&entity.common).map(|m| m.revision).unwrap_or(0);
                    }
                    let next_revision = previous_revision + 1;
                    match state.acquire_lease(handle, previous_revision) {
                        Ok(()) => {
                            if doc.model.modify(handle, |entity| {
                                set_bim_meta(&mut entity.common, &BimEntityMeta {
                                    floor_id,
                                    discipline,
                                    owner_account_id: owner.clone(),
                                    revision: next_revision,
                                });
                            }) {
                                state.queue_update(handle, previous_revision, next_revision);
                            }
                            state.release_lease(handle);
                        }
                        Err(message) => {
                            eprintln!("BIM lock: {message}");
                        }
                    }
                }
            }
        }

        if state.connection == ConnectionMode::Online && state.dirty_local_changes > 0 && ui.button("Đánh dấu đã đồng bộ (mô phỏng server)").clicked() {
            state.mark_all_synced();
        }
        if !state.pending_changes.is_empty() {
            ui.label(format!("ChangeSet local: {} | thay đổi chưa đồng bộ: {}", state.pending_changes.len(), state.dirty_local_changes));
        }
        if !state.leases.is_empty() {
            ui.label(format!("Đối tượng đang khóa mềm: {}", state.leases.len()));
        }
        if !state.conflicts.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("Xung đột cần xử lý: {}", state.conflicts.len()));
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
    ui.collapsing("Tiến độ thiết kế toàn dự án", |ui| {
        let floor_ids = state.floors.iter().map(|f| f.id).collect::<Vec<_>>();
        for floor_id in floor_ids {
            for discipline in Discipline::ALL {
                state.ensure_progress_scope(floor_id, discipline);
            }
        }
        egui::Grid::new("bim_progress").striped(true).show(ui, |ui| {
            ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("%"); ui.strong("Trạng thái"); ui.strong("Cập nhật bởi"); ui.end_row();
            for progress in &mut state.progress {
                ui.label(state.floors.iter().find(|f| f.id == progress.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(progress.discipline.label());
                ui.add(egui::Slider::new(&mut progress.percent, 0..=100));
                let new_status = if progress.percent == 0 { ScopeStatus::NotStarted } else if progress.percent >= 100 { ScopeStatus::Complete } else if progress.status == ScopeStatus::Review { ScopeStatus::Review } else { ScopeStatus::InProgress };
                progress.status = new_status;
                egui::ComboBox::from_id_salt(("scope_status", progress.floor_id, progress.discipline.label()))
                    .selected_text(progress.status.label())
                    .show_ui(ui, |ui| {
                        for status in [ScopeStatus::NotStarted, ScopeStatus::InProgress, ScopeStatus::Review, ScopeStatus::Complete] {
                            ui.selectable_value(&mut progress.status, status, status.label());
                        }
                    });
                ui.label(&progress.updated_by);
                ui.end_row();
            }
        });
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
            let visible_count = state.viewport_entity_count(app, &state.views[i]);
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
                ui.label(format!("Đối tượng phù hợp bộ lọc: {}", visible_count));
                ui.label(if i == 0 { "Viewport làm việc chính" } else { "Viewport phối hợp/xem trực tiếp" });
            });
            if slots > 1 && i % 2 == 1 { ui.end_row(); }
        }
    });

    ui.separator();
    ui.collapsing("Issue phối hợp / Coordination", |ui| {
        let selected = app.session.selection().first().copied();
        ui.horizontal(|ui| {
            if ui.button("+ Issue sang Kết cấu").clicked() { state.create_issue_from_selection(selected, Discipline::Structure); }
            if ui.button("+ Issue sang Điện").clicked() { state.create_issue_from_selection(selected, Discipline::Electrical); }
            if ui.button("+ Issue sang Nước").clicked() { state.create_issue_from_selection(selected, Discipline::Plumbing); }
            if ui.button("+ Issue sang HVAC").clicked() { state.create_issue_from_selection(selected, Discipline::Hvac); }
        });
        egui::Grid::new("bim_issues").striped(true).show(ui, |ui| {
            ui.strong("ID"); ui.strong("Tầng"); ui.strong("Nguồn"); ui.strong("Đích"); ui.strong("Đối tượng"); ui.strong("Phụ trách"); ui.strong("Trạng thái"); ui.end_row();
            for issue in &mut state.issues {
                ui.label(issue.id.to_string());
                ui.label(state.floors.iter().find(|f| f.id == issue.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(issue.source_discipline.label());
                ui.label(issue.target_discipline.label());
                ui.label(issue.handle.map(|h| h.hex()).unwrap_or_else(|| "-".into()));
                ui.text_edit_singleline(&mut issue.assigned_to);
                egui::ComboBox::from_id_salt(("issue_status", issue.id))
                    .selected_text(issue.status.label())
                    .show_ui(ui, |ui| {
                        for status in [IssueStatus::Open, IssueStatus::InProgress, IssueStatus::Resolved] {
                            ui.selectable_value(&mut issue.status, status, status.label());
                        }
                    });
                ui.end_row();
            }
        });
    });

    ui.collapsing("Người đang làm việc / Presence", |ui| {
        egui::Grid::new("bim_presence").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Đang chọn"); ui.end_row();
            for p in &state.presence {
                ui.label(&p.account_id);
                ui.label(state.floors.iter().find(|f| f.id == p.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(p.discipline.label());
                ui.label(p.selected_handle.map(|h| h.hex()).unwrap_or_else(|| "-".into()));
                ui.end_row();
            }
        });
    });

    ui.collapsing("Rule quyền chi tiết theo tầng / bộ môn", |ui| {
        egui::Grid::new("bim_scope_rules").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Xem"); ui.strong("Sửa"); ui.end_row();
            for rule in &mut state.access_rules {
                ui.label(&rule.account_id);
                ui.label(rule.floor_id.and_then(|id| state.floors.iter().find(|f| f.id == id).map(|f| f.name.clone())).unwrap_or_else(|| "Tất cả".into()));
                ui.label(rule.discipline.map(Discipline::label).unwrap_or("Tất cả"));
                ui.checkbox(&mut rule.can_view, "");
                ui.checkbox(&mut rule.can_edit, "");
                ui.end_row();
            }
        });
    });

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
    fn project_progress_auto_creates_scope_and_completion_status() {
        let mut state = State::default();
        state.ensure_progress_scope(2, Discipline::Hvac);
        let p = state.progress.iter_mut().find(|p| p.floor_id == 2 && p.discipline == Discipline::Hvac).unwrap();
        p.percent = 100;
        p.status = ScopeStatus::Complete;
        assert_eq!(p.status, ScopeStatus::Complete);
    }

    #[test]
    fn coordination_issue_is_created_in_active_floor_and_discipline() {
        let mut state = State::default();
        state.active_floor_id = 2;
        state.active_discipline = Discipline::Architecture;
        state.create_issue_from_selection(Some(cadcraft_doc::Handle(12)), Discipline::Structure);
        assert_eq!(state.issues.len(), 1);
        assert_eq!(state.issues[0].floor_id, 2);
        assert_eq!(state.issues[0].source_discipline, Discipline::Architecture);
        assert_eq!(state.issues[0].target_discipline, Discipline::Structure);
    }

    #[test]
    fn detailed_scope_rule_controls_view_and_edit() {
        let mut state = State::default();
        state.access_rules = vec![
            AccessRule { account_id: "u1".into(), floor_id: Some(2), discipline: Some(Discipline::Electrical), can_view: true, can_edit: false }
        ];
        assert!(state.has_scope_access("u1", 2, Discipline::Electrical, false));
        assert!(!state.has_scope_access("u1", 2, Discipline::Electrical, true));
        assert!(!state.has_scope_access("u1", 1, Discipline::Electrical, false));
        assert!(!state.has_scope_access("u1", 2, Discipline::Structure, false));
    }

    #[test]
    fn offline_and_online_login_switch_connection_mode() {
        let mut state = State::default();
        state.login_online_stub("u1".into(), "User One".into(), "u1@example.com".into());
        assert_eq!(state.connection, ConnectionMode::Online);
        assert_eq!(state.auth.status, AuthStatus::OnlineAuthenticated);
        state.login_offline("u2".into(), "User Two".into());
        assert_eq!(state.connection, ConnectionMode::Offline);
        assert_eq!(state.auth.status, AuthStatus::OfflineProfile);
    }

    #[test]
    fn soft_lock_prevents_another_account_from_editing_same_object() {
        let mut state = State::default();
        let handle = cadcraft_doc::Handle(99);
        state.acquire_lease(handle, 1).unwrap();
        state.current_account_id = "struct-01".into();
        assert!(state.acquire_lease(handle, 1).is_err());
    }

    #[test]
    fn presence_tracks_current_user_scope_and_selection() {
        let mut state = State::default();
        state.active_floor_id = 2;
        state.active_discipline = Discipline::Electrical;
        state.update_presence(Some(cadcraft_doc::Handle(7)));
        assert_eq!(state.presence.len(), 1);
        assert_eq!(state.presence[0].floor_id, 2);
        assert_eq!(state.presence[0].discipline, Discipline::Electrical);
        assert_eq!(state.presence[0].selected_handle, Some(cadcraft_doc::Handle(7)));
    }

    #[test]
    fn viewport_filter_matches_floor_discipline_and_optional_owner() {
        let meta = BimEntityMeta { floor_id: 2, discipline: Discipline::Structure, owner_account_id: "struct-01".into(), revision: 1 };
        let mut slot = ViewSlot { floor_id: 2, discipline: Discipline::Structure, owner_account_id: None, overlay_opacity: 0.3, show_other_users: true };
        assert!(slot_matches(&meta, &slot));
        slot.floor_id = 1;
        assert!(!slot_matches(&meta, &slot));
        slot.floor_id = 2;
        slot.owner_account_id = Some("other".into());
        assert!(!slot_matches(&meta, &slot));
    }

    #[test]
    fn revision_mismatch_creates_sync_conflict() {
        let item = ChangeItem { handle: cadcraft_doc::Handle(42), operation: ChangeOperation::Update, base_revision: 4, local_revision: 5 };
        assert!(detect_conflict(&item, 4).is_none());
        let conflict = detect_conflict(&item, 6).unwrap();
        assert_eq!(conflict.base_revision, 4);
        assert_eq!(conflict.local_revision, 5);
        assert_eq!(conflict.remote_revision, 6);
    }

    #[test]
    fn bim_metadata_roundtrips_through_entity_xdata() {
        let mut common = cadcraft_doc::Common::default();
        let meta = BimEntityMeta { floor_id: 7, discipline: Discipline::Electrical, owner_account_id: "user-01".into(), revision: 3 };
        set_bim_meta(&mut common, &meta);
        assert_eq!(bim_meta(&common), Some(meta));
        assert!(common.xdata.iter().any(|t| t.code == 1001 && t.value == XDATA_APP));
    }

    #[test]
    fn offline_changes_are_queued_and_can_be_marked_synced() {
        let mut state = State::default();
        state.queue_update(cadcraft_doc::Handle(10), 2, 3);
        assert_eq!(state.dirty_local_changes, 1);
        assert_eq!(state.pending_changes.len(), 1);
        assert!(!state.pending_changes[0].synced);
        state.mark_all_synced();
        assert_eq!(state.dirty_local_changes, 0);
        assert!(state.pending_changes[0].synced);
    }

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
