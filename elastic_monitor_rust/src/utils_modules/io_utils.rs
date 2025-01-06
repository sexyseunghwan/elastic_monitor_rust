use crate::common::*;

#[doc = "Json 파일을 읽어서 객체로 변환해주는 함수."]
/// # Arguments
/// * `file_path` - 읽을대상 파일이 존재하는 경로
///
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_json_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = from_reader(reader)?;

    Ok(data)
}

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
