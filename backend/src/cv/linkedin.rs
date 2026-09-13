use std::collections::BTreeMap;

use serde_json::Value;

use super::model::{
    Achievement, Certificate, CvData, Education, Experience, Identity, ProfileLink, Project,
    SkillGroup,
};
use crate::error::AppError;

pub fn normalize_profile(profile: &Value) -> Result<CvData, AppError> {
    let object = profile
        .as_object()
        .ok_or_else(|| AppError::BadRequest("LinkedIn profile data must be an object".into()))?;
    let first_name = field(object, &["firstName", "localizedFirstName", "given_name"]);
    let last_name = field(object, &["lastName", "localizedLastName", "family_name"]);
    let full_name = field(object, &["name", "fullName"])
        .or_else(|| join_non_empty([first_name.clone(), last_name.clone()]))
        .unwrap_or_default();
    let vanity = field(object, &["vanityName", "vanity", "publicIdentifier"]);
    let mut profiles = Vec::new();
    if let Some(vanity) = vanity.as_deref().and_then(linkedin_profile_url) {
        profiles.push(ProfileLink {
            id: "linkedin-profile-0".into(),
            r#type: "linkedin".into(),
            label: "LinkedIn".into(),
            url: vanity,
        });
    } else if let Some(url) = field(object, &["profileUrl", "publicProfileUrl"])
        .as_deref()
        .and_then(normalize_url)
    {
        profiles.push(ProfileLink {
            id: "linkedin-profile-0".into(),
            r#type: "linkedin".into(),
            label: "LinkedIn".into(),
            url,
        });
    }
    if let Some(websites) = object.get("websites").and_then(Value::as_array) {
        for (index, website) in websites.iter().enumerate() {
            let url = match website {
                Value::String(value) => normalize_url(value),
                Value::Object(values) => field(values, &["url", "link"])
                    .as_deref()
                    .and_then(normalize_url),
                _ => None,
            };
            if let Some(url) = url {
                if !profiles
                    .iter()
                    .any(|profile: &ProfileLink| profile.url == url)
                {
                    profiles.push(ProfileLink {
                        id: format!("linkedin-website-{index}"),
                        r#type: "website".into(),
                        label: website_label(website).unwrap_or_else(|| "Website".into()),
                        url,
                    });
                }
            }
        }
    }

    let identity = Identity {
        full_name,
        professional_titles: field(object, &["headline", "title"]).unwrap_or_default(),
        location: location(object),
        email: field(object, &["email", "emailAddress", "primaryEmailAddress"])
            .and_then(normalize_email)
            .unwrap_or_default(),
        phone: phone(object),
        profiles,
    };
    Ok(CvData {
        identity,
        summary: field(object, &["summary", "about", "description"]).unwrap_or_default(),
        experience: entries(object, &["positions", "experience"], map_experience),
        achievements: entries(
            object,
            &["honors", "honorsAndAwards", "achievements"],
            map_achievement,
        ),
        skills: map_skills(object),
        education: entries(object, &["education", "educations"], map_education),
        certificates: entries(object, &["certifications", "certificates"], map_certificate),
        projects: entries(object, &["projects"], map_project),
    })
}

fn map_experience(object: &serde_json::Map<String, Value>, index: usize) -> Experience {
    let dates = date_range(object.get("timePeriod").or_else(|| object.get("dateRange")));
    Experience {
        id: format!("linkedin-experience-{index}"),
        role: field(object, &["title", "role", "position"]).unwrap_or_default(),
        organization: field(
            object,
            &["companyName", "company", "organization", "employer"],
        )
        .unwrap_or_default(),
        location: field(object, &["locationName", "location"]).unwrap_or_default(),
        start: date_part(
            object
                .get("start")
                .or_else(|| object.get("startDate"))
                .or_else(|| object.get("timePeriod").and_then(|v| v.get("start"))),
        ),
        end: date_part(
            object
                .get("end")
                .or_else(|| object.get("endDate"))
                .or_else(|| object.get("timePeriod").and_then(|v| v.get("end"))),
        ),
        current: object
            .get("current")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || object
                .get("present")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            || dates
                .as_deref()
                .is_some_and(|value| value.ends_with("Present")),
        description: field(object, &["description", "summary"]).unwrap_or_default(),
        highlights: lines(
            object
                .get("highlights")
                .or_else(|| object.get("accomplishments")),
        ),
        tools: String::new(),
    }
}

fn map_achievement(object: &serde_json::Map<String, Value>, index: usize) -> Achievement {
    Achievement {
        id: format!("linkedin-honor-{index}"),
        title: field(object, &["title", "name"]).unwrap_or_default(),
        category: field(object, &["category", "issuer", "organization"]).unwrap_or_default(),
        date: date_part(object.get("date").or_else(|| object.get("issuedOn"))),
        description: field(object, &["description", "summary"]).unwrap_or_default(),
    }
}

fn map_education(object: &serde_json::Map<String, Value>, index: usize) -> Education {
    Education {
        id: format!("linkedin-education-{index}"),
        institution: field(object, &["schoolName", "institution", "school"]).unwrap_or_default(),
        qualification: join_non_empty([
            field(object, &["degreeName", "degree"]),
            field(object, &["fieldOfStudy", "field"]),
        ])
        .unwrap_or_default(),
        location: field(object, &["locationName", "location"]).unwrap_or_default(),
        start: date_part(
            object
                .get("start")
                .or_else(|| object.get("startDate"))
                .or_else(|| object.get("timePeriod").and_then(|v| v.get("start"))),
        ),
        end: date_part(
            object
                .get("end")
                .or_else(|| object.get("endDate"))
                .or_else(|| object.get("timePeriod").and_then(|v| v.get("end"))),
        ),
        gpa: field(object, &["gpa", "grade"]).unwrap_or_default(),
    }
}

fn map_certificate(object: &serde_json::Map<String, Value>, index: usize) -> Certificate {
    Certificate {
        id: format!("linkedin-certificate-{index}"),
        name: field(object, &["name", "title"]).unwrap_or_default(),
        issuer: field(object, &["authority", "issuer", "organization"]).unwrap_or_default(),
        date: date_part(
            object
                .get("date")
                .or_else(|| object.get("issuedOn"))
                .or_else(|| object.get("timePeriod").and_then(|v| v.get("start"))),
        ),
        credential_url: field(object, &["url", "credentialUrl", "credential_url"])
            .as_deref()
            .and_then(normalize_url)
            .unwrap_or_default(),
    }
}

fn map_project(object: &serde_json::Map<String, Value>, index: usize) -> Project {
    Project {
        id: format!("linkedin-project-{index}"),
        name: field(object, &["name", "title"]).unwrap_or_default(),
        role: field(object, &["role"]).unwrap_or_default(),
        url: field(object, &["url", "projectUrl", "link"])
            .as_deref()
            .and_then(normalize_url)
            .unwrap_or_default(),
        dates: date_range(object.get("timePeriod").or_else(|| object.get("dateRange")))
            .unwrap_or_default(),
        description: field(object, &["description", "summary"]).unwrap_or_default(),
        highlights: lines(object.get("highlights")),
        tools: String::new(),
    }
}

fn map_skills(object: &serde_json::Map<String, Value>) -> Vec<SkillGroup> {
    let Some(values) = object.get("skills").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for value in values {
        let Some(item) = value.as_object() else {
            continue;
        };
        let skill = field(item, &["name", "skill", "title"]);
        if let Some(skill) = skill.filter(|value| !value.is_empty()) {
            let category = field(item, &["category", "type"]).unwrap_or_else(|| "Skills".into());
            groups.entry(category).or_default().push(skill);
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (category, skills))| SkillGroup {
            id: format!("linkedin-skills-{index}"),
            category,
            skills: skills.join(", "),
        })
        .collect()
}

fn entries<T>(
    object: &serde_json::Map<String, Value>,
    keys: &[&str],
    map: fn(&serde_json::Map<String, Value>, usize) -> T,
) -> Vec<T> {
    keys.iter()
        .find_map(|key| object.get(*key).and_then(Value::as_array))
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_object)
                .enumerate()
                .map(|(index, value)| map(value, index))
                .collect()
        })
        .unwrap_or_default()
}

fn field(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| object.get(*key))
        .map(flatten_rich)
        .map(|value| value.trim().to_owned())
        .find(|value| !value.is_empty())
}

fn location(object: &serde_json::Map<String, Value>) -> String {
    field(object, &["location", "locationName", "geoLocationName"]).unwrap_or_else(|| {
        [
            field(object, &["city"]),
            field(object, &["region", "state"]),
            field(object, &["country", "countryName"]),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
    })
}

fn phone(object: &serde_json::Map<String, Value>) -> String {
    field(object, &["phone", "phoneNumber"]).unwrap_or_else(|| {
        object
            .get("phoneNumbers")
            .and_then(Value::as_array)
            .and_then(|values| {
                values.iter().find_map(|value| {
                    value
                        .as_object()
                        .and_then(|item| field(item, &["number", "phoneNumber"]))
                })
            })
            .unwrap_or_default()
    })
}

fn website_label(value: &Value) -> Option<String> {
    value
        .as_object()
        .and_then(|object| field(object, &["label", "name", "title"]))
}

fn join_non_empty<const N: usize>(values: [Option<String>; N]) -> Option<String> {
    let values = values
        .into_iter()
        .flatten()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(" "))
}

fn lines(value: Option<&Value>) -> String {
    match value {
        Some(Value::Array(values)) => values
            .iter()
            .map(flatten_rich)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Some(value) => flatten_rich(value),
        None => String::new(),
    }
}

fn date_range(value: Option<&Value>) -> Option<String> {
    let object = value?.as_object()?;
    let start = date_part(object.get("start").or_else(|| object.get("from")));
    let end = date_part(object.get("end").or_else(|| object.get("to")));
    let current = object
        .get("present")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || object
            .get("current")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    join_non_empty([
        Some(start),
        Some(if current { "Present".into() } else { end }),
    ])
    .filter(|value| !value.is_empty())
}

fn date_part(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    if let Some(object) = value.as_object() {
        let year = object.get("year").and_then(number_text);
        let month = object.get("month").and_then(number_text);
        return match (year, month) {
            (Some(year), Some(month)) if month.len() <= 2 => format!("{year}-{:0>2}", month),
            (Some(year), None) => year,
            _ => field(object, &["date", "value"])
                .map(|value| normalize_date_text(&value))
                .unwrap_or_default(),
        };
    }
    normalize_date_text(&flatten_rich(value))
}

fn number_text(value: &Value) -> Option<String> {
    value
        .as_i64()
        .map(|value| value.to_string())
        .or_else(|| value.as_str().map(|value| value.trim().to_owned()))
}

fn normalize_date_text(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 7 && value.as_bytes().get(4) == Some(&b'-') {
        if value.as_bytes().get(7) == Some(&b'-') {
            return value[..7].to_owned();
        }
        if value[5..7]
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            return value[..7].to_owned();
        }
    }
    if value.len() >= 4
        && value[..4]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return value[..4].to_owned();
    }
    value.to_owned()
}

fn normalize_url(value: &str) -> Option<String> {
    let value = value.trim();
    let candidate = if value.starts_with("http://") || value.starts_with("https://") {
        value.to_owned()
    } else {
        format!("https://{value}")
    };
    let parsed = url::Url::parse(&candidate).ok()?;
    matches!(parsed.scheme(), "http" | "https").then(|| parsed.to_string())
}

fn linkedin_profile_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        return normalize_url(value);
    }
    Some(format!(
        "https://www.linkedin.com/in/{}",
        value.trim_start_matches("www.linkedin.com/in/")
    ))
}

fn normalize_email(value: String) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    (value.len() >= 3
        && value.len() <= 320
        && value.bytes().filter(|byte| *byte == b'@').count() == 1
        && !value.starts_with('@')
        && !value.ends_with('@'))
    .then_some(value)
}

fn flatten_rich(value: &Value) -> String {
    let raw = match value {
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(flatten_rich)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => {
            let localized = object.get("localized").and_then(|localized| {
                let preferred = object
                    .get("preferredLocale")
                    .and_then(Value::as_object)
                    .and_then(|locale| {
                        Some(format!(
                            "{}_{}",
                            locale.get("language")?.as_str()?,
                            locale.get("country")?.as_str()?
                        ))
                    });
                preferred
                    .as_deref()
                    .and_then(|key| localized.as_object()?.get(key))
                    .or_else(|| localized.as_object()?.values().next())
                    .map(flatten_rich)
            });
            localized
                .or_else(|| {
                    ["text", "content", "html", "value", "name"]
                        .iter()
                        .find_map(|key| object.get(*key).map(flatten_rich))
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    };
    strip_html(&raw)
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_partner_profile_fields_to_normalized_cv_data() {
        let profile = serde_json::json!({
            "localizedFirstName": "Ada",
            "localizedLastName": "Lovelace",
            "headline": "Mathematician",
            "city": "London",
            "country": "UK",
            "vanityName": "ada-lovelace",
            "emailAddress": "ada@example.com",
            "phoneNumbers": [{"number": "+44 20 1234"}],
            "websites": [{"label": "Portfolio", "url": "www.example.com"}],
            "summary": "<p>Built &amp; explained systems.</p>",
            "positions": [{"title": "Engineer", "companyName": "Analytical Engines", "timePeriod": {"start": {"year": 1842, "month": 1}, "present": true}, "description": {"html": "<b>Designed</b> engines"}}],
            "education": [{"schoolName": "University", "degreeName": "BA", "fieldOfStudy": "Math", "startDate": "1830-09-01", "endDate": "1835-06-01"}],
            "skills": [{"name": "Rust"}, {"name": "Systems"}],
            "certifications": [{"name": "Certificate", "authority": "Institute", "issuedOn": {"year": 2020}, "url": "https://example.com/cert"}],
            "projects": [{"name": "Engine", "url": "example.com/project", "description": "A project"}],
            "honors": [{"name": "Prize", "issuer": "Academy", "date": "2024-04-01"}]
        });
        let data = normalize_profile(&profile).unwrap();
        assert_eq!(data.identity.full_name, "Ada Lovelace");
        assert_eq!(data.identity.location, "London, UK");
        assert_eq!(data.summary, "Built & explained systems.");
        assert_eq!(data.experience[0].start, "1842-01");
        assert!(data.experience[0].current);
        assert_eq!(data.education[0].end, "1835-06");
        assert_eq!(data.projects[0].url, "https://example.com/project");
        assert_eq!(data.achievements[0].title, "Prize");
        let serialized = serde_json::to_value(&data).unwrap();
        assert_eq!(serialized["identity"]["fullName"], "Ada Lovelace");
        assert_eq!(
            serialized["certificates"][0]["credentialUrl"],
            "https://example.com/cert"
        );
        assert!(
            data.identity
                .profiles
                .iter()
                .any(|profile| profile.r#type == "website")
        );
    }
}
