use anyhow::Result;
use clap::Parser;

use archeo_gui::ai;
use archeo_gui::cli::{BuiltinProfile, Cli, Commands};
use archeo_gui::registry;
use archeo_gui::scanner;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { root, profile, db } => {
            let db_path = db.unwrap_or_else(|| {
                root.join(".archeo")
                    .join("archeo.sqlite")
            });

            let config =
                scanner::config::ScanConfig::from_optional_yaml_file(
                    profile.as_ref(),
                )?;

            let entries =
                scanner::scan::scan_folder(&root, &config)?;

            let mut registry =
                registry::db::RegistryDb::open(&db_path)?;

            let scan_run =
                registry.insert_scan_run(&root, &config)?;

            registry.insert_file_entries(
                &scan_run.id,
                entries,
            )?;

            println!("scan complete");
            println!("root: {}", root.display());
            println!("profile: {}", config.name);
            println!("database: {}", db_path.display());
        }

        Commands::Cluster {
            db,
            model,
            ollama_url,
            root,
            timeout_seconds,
        } => {
            let db_path = match (root, db) {
                (Some(root), None) => {
                    root.join(".archeo")
                        .join("archeo.sqlite")
                }

                (None, Some(db)) => db,

                (None, None) => {
                    anyhow::bail!(
                        "either --root or --db must be supplied"
                    );
                }

                (Some(_), Some(_)) => {
                    unreachable!();
                }
            };

            let mut registry =
                registry::db::RegistryDb::open(&db_path)?;

            let scan_run =
                registry.latest_scan_run()?;

            let buckets =
                registry.naive_file_buckets_for_scan(
                    &scan_run.id,
                    3,
                )?;

            println!(
                "created {} clustering buckets",
                buckets.len()
            );
            
            let total_files = buckets.iter().map(|bucket| bucket.len()).sum::<usize>() as f64;

            let min_bucket_size = (total_files * 0.05).ceil() as usize;
            let min_bucket_size = min_bucket_size.max(2);

            let common_buckets: Vec<_> = buckets
                .into_iter()
                .filter(|bucket| {
                    bucket.len() >= min_bucket_size
                })
                .collect();

            let prompt =
                ai::cluster_prompt::build_cluster_prompt(
                    &scan_run,
                    &common_buckets,
                )?;

            println!(
                "using {} buckets for first-pass clustering",
                common_buckets.len()
            );


            let prompt_snapshot_id =
                registry.insert_prompt_snapshot(
                    &scan_run.id,
                    &model,
                    &prompt,
                )?;

            let ollama =
                ai::ollama::OllamaClient::new(
                    ollama_url,
                    model.clone(),
                    timeout_seconds,
                );

            let response_text =
                ollama.generate(&prompt)?;

            let response =
                ai::cluster_response::parse_cluster_response(
                    &response_text,
                )?;

            registry.insert_cluster_response(
                &scan_run.id,
                &model,
                &prompt_snapshot_id,
                response,
            )?;

            println!("cluster complete");
            println!("database: {}", db_path.display());
            println!("model: {model}");
        }

        Commands::InitProfile {
            profile,
            out,
        } => {
            match profile {
                BuiltinProfile::Default => {
                    scanner::config::ScanConfig::write_default_profile(&out)?;
                }

                BuiltinProfile::SingleCell => {
                    scanner::config::ScanConfig::write_single_cell_profile(&out)?;
                }
            }

            println!(
                "profile written: {}",
                out.display()
            );
        }
    }

    Ok(())
}
