use std::{fs::create_dir_all, path::PathBuf};

use directories::ProjectDirs;

const FILE_NAME: &str = "todos.db";

#[derive(thiserror::Error, Debug)]
pub enum GetDbPathError {
    #[error("Failed to get the database path")]
    GetDbPath,

    #[error("Failed to create the directory")]
    CreateDir(#[from] std::io::Error),
}

pub fn get_db_path() -> Result<PathBuf, GetDbPathError> {
    if let Some(project) = ProjectDirs::from("com", "dely", "todo") {
        let config_dir = project.config_dir();
        create_dir_all(config_dir)?;
        return Ok(config_dir.join(FILE_NAME));
    }

    Err(GetDbPathError::GetDbPath)
}

