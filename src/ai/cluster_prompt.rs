use anyhow::Result;

use crate::registry::models::{FileBucket, ScanRun};

pub fn build_cluster_prompt(
    scan_run: &ScanRun,
    buckets: &[FileBucket],
) -> Result<String> {
    let mut prompt = String::new();

    prompt.push_str(
        "You are helping organize a scientific research folder.\n\n",
    );

    prompt.push_str(
        "The filesystem has already been summarized into file buckets.\n",
    );

    prompt.push_str(
        "Infer semantic artifact groups from these summaries.\n\n",
    );

    prompt.push_str(
        "Return JSON only.\n",
    );

    prompt.push_str(
        "Do not explain your reasoning.\n",
    );

    prompt.push_str(
        "Do not wrap the JSON in markdown.\n\n",
    );

    prompt.push_str(
        r#"
Return exactly this schema:

{
  "groups": [
    {
      "name": "Analysis notebooks",
      "group_kind": "notebooks",
      "confidence": 0.85,
      "ai_statement": "Notebook-based analysis workflow.",
      "members": [],
      "assumptions": [
        "The notebooks likely represent analysis workflows."
      ]
    }
  ]
}

Allowed group_kind values:
- raw_data
- analysis_scripts
- notebooks
- processed_data
- result_files
- figures
- logs
- archives
- temporary
- publication
- mixed
- unknown
"#,
    );

    prompt.push_str("\n\n");

    prompt.push_str(&format!(
        "Root folder:\n{}\n\n",
        scan_run.root_path
    ));

    prompt.push_str("File buckets:\n\n");

    for bucket in buckets {
        prompt.push_str(&format!("{bucket}\n\n"));
    }
    println!("build_cluster_prompt created the string:\n{prompt}");
    Ok(prompt)
}
