use crate::domain::RoleCode;

#[derive(Debug, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "RoleCodeMapping", rename_all = "PascalCase")]
pub enum RoleCodeMapping {
    Admin,
    Editor,
    Viewer,
}

impl From<RoleCodeMapping> for RoleCode {
    // ---
    fn from(m: RoleCodeMapping) -> Self {
        match m {
            RoleCodeMapping::Admin => RoleCode::Admin,
            RoleCodeMapping::Editor => RoleCode::Editor,
            RoleCodeMapping::Viewer => RoleCode::Viewer,
        }
    }
}

impl From<RoleCode> for RoleCodeMapping {
    // ---
    fn from(m: RoleCode) -> Self {
        match m {
            RoleCode::Admin => RoleCodeMapping::Admin,
            RoleCode::Editor => RoleCodeMapping::Editor,
            RoleCode::Viewer => RoleCodeMapping::Viewer,
        }
    }
}
