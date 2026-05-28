use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::registry::models::ArtifactGroupKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResponse {
    pub groups: Vec<ClusterGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterGroup {
    pub name: String,

    pub group_kind: ArtifactGroupKind,

    pub confidence: f32,

    pub ai_statement: String,

    #[serde(default)]
    pub members: Vec<ClusterMember>,

    #[serde(default)]
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMember {
    pub rel_path: String,

    pub role: Option<String>,
}

pub fn parse_cluster_response(
    text: &str,
) -> Result<ClusterResponse> {
    let cleaned = extract_json_block(text);

    let response: ClusterResponse =
        serde_json::from_str(cleaned).with_context(|| {
            format!(
                "failed to parse cluster response json\n\n{}",
                cleaned
            )
        })?;

    Ok(response)
}

fn extract_json_block(text: &str) -> &str {
    let trimmed = text.trim();

    if trimmed.starts_with("```json") {
        return trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
    }

    if trimmed.starts_with("```") {
        return trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_json_response() {
        let json = r#"
{
  "groups": [
    {
      "name": "Notebooks",
      "group_kind": "notebooks",
      "confidence": 0.92,
      "ai_statement": "Notebook workflow files",
      "members": [
        {
          "rel_path": "analysis.ipynb",
          "role": "script"
        }
      ],
      "assumptions": [
        "Likely main workflow"
      ]
    }
  ]
}
"#;

        let parsed =
            parse_cluster_response(json).unwrap();

        assert_eq!(parsed.groups.len(), 1);

        let group = &parsed.groups[0];

        assert_eq!(group.name, "Notebooks");

        assert_eq!(
            group.group_kind,
            ArtifactGroupKind::Notebooks
        );

        assert_eq!(group.members.len(), 1);

        assert_eq!(
            group.members[0].rel_path,
            "analysis.ipynb"
        );
    }

    #[test]
    fn parses_markdown_wrapped_json() {
        let json = r#"
```json
{
  "groups": []
}
```
"#;

        let parsed =
            parse_cluster_response(json).unwrap();

        assert!(parsed.groups.is_empty());
    }

    #[test]
    fn invalid_json_returns_error() {
        let result =
            parse_cluster_response("not json");

        assert!(result.is_err());
    }

    #[test]
    fn assumptions_default_to_empty() {
        let json = r#"
{
  "groups": [
    {
      "name": "Data",
      "group_kind": "raw_data",
      "confidence": 0.5,
      "ai_statement": "Raw data files"
    }
  ]
}
"#;

        let parsed =
            parse_cluster_response(json).unwrap();

        assert!(
            parsed.groups[0]
                .assumptions
                .is_empty()
        );
    }

    #[test]
    fn members_default_to_empty() {
        let json = r#"
{
  "groups": [
    {
      "name": "Data",
      "group_kind": "raw_data",
      "confidence": 0.5,
      "ai_statement": "Raw data files"
    }
  ]
}
"#;

        let parsed =
            parse_cluster_response(json).unwrap();

        assert!(
            parsed.groups[0]
                .members
                .is_empty()
        );
    }

    #[test]
    fn extract_json_block_removes_code_fences() {
        let text = r#"
```json
{
  "groups": []
}
```
"#;

        let cleaned = extract_json_block(text);

        assert_eq!(
            cleaned,
            "{\n  \"groups\": []\n}"
        );
    }
}

