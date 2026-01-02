use std::fs;

use crate::common::*;

#[doc = "toml 파일을 읽어서 객체로 변환해주는 함수"]
/// # Arguments
/// * `file_path` - 읽을 대상 toml 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_toml_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    let toml_content = std::fs::read_to_string(file_path)?;
    let toml: T = toml::from_str(&toml_content)?;

    Ok(toml)
}

#[doc = "Function that removes a specific file."]
pub fn delete_file(file_path: &PathBuf) -> anyhow::Result<()> {
    fs::remove_file(file_path).map_err(|e| anyhow!("[delete_file] {:?}", e))?;
    Ok(())
}

#[doc = "Function that removes a specific files."]
pub fn delete_files_if_exists<I>(file_paths: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = PathBuf>,
{
    for path in file_paths {
        if path.exists() {
            match delete_file(&path) {
                Ok(_) => {
                    info!("[io_utiles::delete_files_if_exists] The deletion of the `{:?}` file was successful", path);
                }
                Err(e) => {
                    error!("[io_utiles::delete_files_if_exists] {:?}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}

#[doc = "Function that removes all files in a directory."]
/// # Arguments
/// * `dir_path` - The directory path to clean up
///
/// # Returns
/// * `Ok(())` - If all files were deleted successfully or directory doesn't exist
/// * `Err` - If there was an error reading the directory
pub fn delete_all_files_in_directory(dir_path: &str) -> anyhow::Result<()> {
    let path: PathBuf = PathBuf::from(dir_path);

    if !path.exists() {
        info!(
            "[io_utils::delete_all_files_in_directory] Directory does not exist: {:?}",
            dir_path
        );
        return Ok(());
    }

    if !path.is_dir() {
        return Err(anyhow!(
            "[io_utils::delete_all_files_in_directory] Path is not a directory: {:?}",
            dir_path
        ));
    }

    let entries: fs::ReadDir = fs::read_dir(&path).map_err(|e| {
        anyhow!(
            "[io_utils::delete_all_files_in_directory] Failed to read directory: {:?}",
            e
        )
    })?;

    let mut deleted_count: i32 = 0;
    let mut error_count: i32 = 0;

    for entry in entries {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    match fs::remove_file(&entry_path) {
                        Ok(_) => {
                            deleted_count += 1;
                            info!(
                                "[io_utils::delete_all_files_in_directory] Deleted file: {:?}",
                                entry_path
                            );
                        }
                        Err(e) => {
                            error_count += 1;
                            error!("[io_utils::delete_all_files_in_directory] Failed to delete file {:?}: {:?}", entry_path, e);
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                error!(
                    "[io_utils::delete_all_files_in_directory] Failed to read entry: {:?}",
                    e
                );
            }
        }
    }

    info!(
        "[io_utils::delete_all_files_in_directory] Cleanup completed. Deleted: {}, Errors: {}",
        deleted_count, error_count
    );

    Ok(())
}
