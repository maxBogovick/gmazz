use clap::{Parser, Subcommand, Args};
use crate::AppState;
use std::sync::Arc;
use crate::auth::hash_api_key;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage API keys
    Key(KeyArgs),
    /// Run health checks
    Doctor,
}

#[derive(Args)]
pub struct KeyArgs {
    #[command(subcommand)]
    pub command: KeyCommands,
}

#[derive(Subcommand)]
pub enum KeyCommands {
    Create {
        #[arg(short, long)]
        name: String,
    },
    Activate {
        #[arg(short, long)]
        id: String,
    },
    Revoke {
        #[arg(short, long)]
        id: String,
    },
    List,
}

pub async fn handle_cli(state: Arc<AppState>, cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Some(Commands::Key(args)) => match args.command {
            KeyCommands::Create { name } => {
                let plain_key = Uuid::new_v4().to_string().replace("-", "") + &Uuid::new_v4().to_string().replace("-", "");
                let hash = hash_api_key(&state.config.security.server_secret, &plain_key);
                
                let id = state.repo.create_app(&name, &hash).await?;
                println!("App created successfully!");
                println!("Name: {}", name);
                println!("ID: {}", id);
                println!("API KEY: {}", plain_key);
                println!("(Save this key! It will not be shown again.)");
            },
            KeyCommands::List => {
                #[derive(sqlx::FromRow)]
                struct AppRow {
                    id: String,
                    name: Option<String>,
                    is_active: bool,
                }

                let apps = sqlx::query_as::<_, AppRow>("SELECT id, name, is_active FROM apps")
                    .fetch_all(&state.repo.pool).await?;
                
                println!("{:<36} | {:<20} | {:<10}", "ID", "Name", "Active");
                println!("{}", "-".repeat(70));
                for app in apps {
                    println!("{:<36} | {:<20} | {:<10}", 
                        app.id, 
                        app.name.unwrap_or_default(), 
                        app.is_active);
                }
            },
            KeyCommands::Revoke { id } => {
                sqlx::query("UPDATE apps SET is_active = 0 WHERE id = ?")
                    .bind(id.clone())
                    .execute(&state.repo.pool).await?;
                println!("App {} revoked.", id);
            },
            KeyCommands::Activate { id } => {
                sqlx::query("UPDATE apps SET is_active = 1 WHERE id = ?")
                    .bind(id.clone())
                    .execute(&state.repo.pool).await?;
                println!("App {} activated.", id);
            }
        },
        Some(Commands::Doctor) => {
            println!("Starting system integrity check...");
            
            // 1. Check DB -> Disk
            #[derive(sqlx::FromRow)]
            struct FileRow {
                id: String,
                original_name: String,
                stored_path: String,
            }

            let files = sqlx::query_as::<_, FileRow>("SELECT id, original_name, stored_path FROM files WHERE deleted_at IS NULL")
                .fetch_all(&state.repo.pool).await?;
            
            let mut missing = 0;
            for file in &files {
                let abs_path = state.storage.get_absolute_path(&file.stored_path);
                if !abs_path.exists() {
                    println!("[ERR] File in DB but missing on disk: ID={}, Name={}, Path={}", file.id, file.original_name, file.stored_path);
                    missing += 1;
                }
            }

            // 2. Check Disk -> DB (Orphans)
            let files_dir = state.config.storage.data_dir.join("files");
            let mut orphans = 0;
            if files_dir.exists() {
                use walkdir::WalkDir;
                for entry in WalkDir::new(&files_dir).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        // Skip sqlite files
                        if entry.path().extension().and_then(|s| s.to_str()) == Some("db") { continue; }

                        let rel_path = entry.path().strip_prefix(&state.config.storage.data_dir)
                            .unwrap().to_string_lossy().to_string();
                        
                        let exists_in_db = sqlx::query("SELECT id FROM files WHERE stored_path = ?")
                            .bind(&rel_path)
                            .fetch_optional(&state.repo.pool).await?;
                        
                        if exists_in_db.is_none() {
                            println!("[WARN] Orphaned file on disk (not in DB): {}", rel_path);
                            orphans += 1;
                        }
                    }
                }
            }

            println!("--- Summary ---");
            println!("Files in DB: {}", files.len());
            println!("Missing on disk: {}", missing);
            println!("Orphaned on disk: {}", orphans);
            if missing == 0 && orphans == 0 {
                println!("System is healthy! OK");
            } else {
                println!("Integrity issues found.");
            }
        },
        None => {}
    }
    Ok(())
}
