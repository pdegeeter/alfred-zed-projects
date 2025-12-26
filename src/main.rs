use dirs::config_dir;
use serde::Serialize;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

mod constants;
use crate::constants::*;

#[derive(Debug)]
struct Workspace {
    workspace_id: i64,
    local_paths: String,
}

#[derive(Serialize, Debug)]
struct Item {
    uid: String,
    title: String,
    subtitle: String,
    icon: Icon,
    arg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    valid: Option<bool>,
}

impl From<Workspace> for Item {
    fn from(workspace: Workspace) -> Self {
        let path = workspace.local_paths;
        let index = path.find('/').unwrap_or(path.len());
        let path = path.split_at(index).1;
        let name = path.trim_end_matches('/').split('/').last().unwrap_or("");
        Self {
            uid: workspace.workspace_id.to_string(),
            title: name.to_owned(),
            subtitle: path.to_owned(),
            icon: Icon::new(path.to_owned()),
            arg: path.to_owned(),
            valid: None,
        }
    }
}

impl Item {
    fn from_path(path: PathBuf) -> Option<Self> {
        let path_str = path.to_str()?.to_owned();
        let name = path.file_name()?.to_str()?.to_owned();
        Some(Self {
            uid: path_str.clone(),
            title: name,
            subtitle: path_str.clone(),
            icon: Icon::new(path_str.clone()),
            arg: path_str,
            valid: None,
        })
    }
}

#[derive(Serialize, Debug)]
struct Icon {
    path: String,
    r#type: String,
}

impl Icon {
    fn new(path: String) -> Icon {
        Icon {
            path,
            r#type: ICON_TYPE_FILE.to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
struct Response {
    items: Vec<Item>,
}

fn list_recent_projects(query: Option<String>) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let path = config_dir().unwrap().to_str().unwrap().to_owned() + DB_PATH;
    let connection = sqlite::open(path)?;
    let stmt = connection.prepare(format!(
        "SELECT {DB_FIELD_WORKSPACE_ID}, {DB_FIELD_PATHS}
            FROM workspaces
            WHERE {DB_FIELD_PATHS} IS NOT NULL
            ORDER BY timestamp DESC"
    ))?;

    let mut items: Vec<Item> = stmt
        .into_iter()
        .filter_map(|row| row.ok())
        .map(|row| {
            let workspace = Workspace {
                workspace_id: row.read::<i64, _>(DB_FIELD_WORKSPACE_ID),
                local_paths: row.read::<&str, _>(DB_FIELD_PATHS).to_string(),
            };
            Item::from(workspace)
        })
        .collect();

    // Filter by query
    if let Some(query) = query {
        let query_lower = query.to_lowercase();
        items.retain(|item| {
            item.title.to_lowercase().contains(&query_lower)
                || item.subtitle.to_lowercase().contains(&query_lower)
        });
    }

    Ok(items)
}

fn list_directories(query: Option<String>) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let dirs_env = env::var("projects_directories").unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();

    let mut items: Vec<Item> = Vec::new();

    for line in dirs_env.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Expand ~ to home directory
        let dir_path = if line.starts_with('~') {
            line.replacen('~', &home, 1)
        } else {
            line.to_string()
        };

        let path = PathBuf::from(&dir_path);
        if !path.is_dir() {
            continue;
        }

        // Read directory entries
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Skip hidden directories
                    if let Some(name) = entry_path.file_name() {
                        if name.to_str().map(|s| s.starts_with('.')).unwrap_or(false) {
                            continue;
                        }
                    }
                    if let Some(item) = Item::from_path(entry_path) {
                        items.push(item);
                    }
                }
            }
        }
    }

    // Sort by title
    items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    // Filter by query
    if let Some(query) = query {
        let query_lower = query.to_lowercase();
        items.retain(|item| {
            item.title.to_lowercase().contains(&query_lower)
                || item.subtitle.to_lowercase().contains(&query_lower)
        });
    }

    Ok(items)
}

fn no_results_item() -> Item {
    Item {
        uid: "no-results".to_string(),
        title: "No results found".to_string(),
        subtitle: "Try a different search term".to_string(),
        icon: Icon {
            path: "icon.png".to_string(),
            r#type: "".to_string(),
        },
        arg: "".to_string(),
        valid: Some(false),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let (mode, query) = if args.len() > 1 && args[1] == "--dirs" {
        ("dirs", args.get(2).cloned())
    } else {
        ("recent", args.get(1).cloned())
    };

    let mut items = match mode {
        "dirs" => list_directories(query)?,
        _ => list_recent_projects(query)?,
    };

    if items.is_empty() {
        items.push(no_results_item());
    }

    serde_json::to_writer(io::stdout(), &Response { items })?;
    Ok(())
}
