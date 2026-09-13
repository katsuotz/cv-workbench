use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

fn default_schema_version() -> i32 {
    1
}

fn default_template_id() -> String {
    "editorial-v1".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct SaveCvSessionRequest {
    pub project_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    #[serde(default = "default_schema_version")]
    pub schema_version: i32,
    #[serde(default = "default_template_id", alias = "templateId")]
    pub template_id: String,
    #[serde(default, alias = "generatedTemplateId")]
    pub generated_template_id: Option<String>,
    pub data: Value,
    pub generated_source: Option<String>,
    pub generated_at: Option<String>,
    pub fingerprint: Option<String>,
    #[serde(default, alias = "expected_version", alias = "base_version")]
    pub version: i64,
}

#[derive(Clone, Debug)]
pub struct SaveCvSessionInput {
    pub project_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub schema_version: i32,
    pub template_id: String,
    pub generated_template_id: Option<String>,
    pub data: Value,
    pub generated_source: Option<String>,
    pub generated_at: Option<OffsetDateTime>,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectMetadata {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RevisionMetadata {
    pub id: Uuid,
    pub number: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentMetadata {
    pub id: Uuid,
    pub name: String,
    pub updated_at: String,
    pub latest_revision: Option<RevisionMetadata>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CvSessionResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub document_id: Uuid,
    pub project: ProjectMetadata,
    pub document: DocumentMetadata,
    pub latest_revision_id: Option<Uuid>,
    pub latest_revision_number: Option<i64>,
    pub latest_revision_created_at: Option<String>,
    pub schema_version: i32,
    pub template_id: String,
    pub generated_template_id: Option<String>,
    pub data: Value,
    pub generated_source: Option<String>,
    pub generated_at: Option<String>,
    pub fingerprint: Option<String>,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingCvImportResponse {
    pub id: Uuid,
    pub data: CvData,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CvData {
    #[serde(default)]
    pub identity: Identity,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub experience: Vec<Experience>,
    #[serde(default)]
    pub achievements: Vec<Achievement>,
    #[serde(default)]
    pub skills: Vec<SkillGroup>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub certificates: Vec<Certificate>,
    #[serde(default)]
    pub projects: Vec<Project>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(default, rename = "fullName", alias = "full_name")]
    pub full_name: String,
    #[serde(default, rename = "professionalTitles", alias = "professional_titles")]
    pub professional_titles: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub profiles: Vec<ProfileLink>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileLink {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Experience {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub organization: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    #[serde(default)]
    pub current: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub highlights: String,
    #[serde(default)]
    pub tools: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkillGroup {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub skills: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Education {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub institution: String,
    #[serde(default)]
    pub qualification: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    #[serde(default)]
    pub gpa: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub date: String,
    #[serde(default, rename = "credentialUrl", alias = "credential_url")]
    pub credential_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub dates: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub highlights: String,
    #[serde(default)]
    pub tools: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RenderCvRequest {
    #[serde(default = "default_template_id", alias = "templateId")]
    pub template_id: String,
    #[serde(alias = "cv_data", alias = "cvData")]
    pub data: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct TemplateCatalogItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub display_order: i32,
    pub active: bool,
    pub preview_asset: String,
    pub preview_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CvTemplateRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub display_order: i32,
    pub source_asset: String,
    pub preview_asset: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RenderCvResponse {
    pub template_id: String,
    pub source: String,
    pub generated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_optimistic_version_aliases_and_defaults() {
        let request: SaveCvSessionRequest = serde_json::from_value(serde_json::json!({
            "data": {"identity": {}},
            "expected_version": 4
        }))
        .unwrap();
        assert_eq!(request.version, 4);
        assert_eq!(request.schema_version, 1);
        assert_eq!(request.template_id, "editorial-v1");

        let legacy: SaveCvSessionRequest = serde_json::from_value(serde_json::json!({
            "data": {"identity": {}},
            "template_id": "default"
        }))
        .unwrap();
        assert_eq!(legacy.template_id, "default");
    }
}
