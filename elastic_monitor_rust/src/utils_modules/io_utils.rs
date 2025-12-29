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
                Ok(_) => (),
                Err(e) => {
                    error!("[io_utiles -> delete_files_if_exists] {:?}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}
