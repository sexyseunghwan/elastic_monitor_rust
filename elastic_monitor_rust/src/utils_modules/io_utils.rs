use crate::common::*;


/*
    Json 파일을 읽어서 객체로 변환해주는 함수
*/
pub fn read_json_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = from_reader(reader)?;
    
    Ok(data)
}