use crate::common::*;

#[doc = "json 객체를 파싱하기 위한 함수"]
pub fn get_value_by_path<T: FromStr>(json_value: &Value, path: &str) -> Result<T, anyhow::Error> {
    let mut current_value = json_value;

    // 경로를 "."으로 분리하여 탐색
    for key in path.split('.') {
        current_value = current_value.get(key).ok_or_else(|| {
            anyhow!(
                "[get_value_by_path()][Parsing Error] Path '{}' not found.",
                path
            )
        })?;
    }

    // 최종 값을 원하는 타입으로 변환
    match current_value {
        Value::String(s) => s.parse::<T>().map_err(|_| {
            anyhow!(
                "[get_value_by_path()][Parsing Error] '{}' cannot be parsed to target type.",
                path
            )
        }),
        Value::Number(n) => n.to_string().parse::<T>().map_err(|_| {
            anyhow!(
                "[get_value_by_path()][Parsing Error] '{}' cannot be parsed to target type.",
                path
            )
        }),
        _ => Err(anyhow!(
            "[get_value_by_path()][Parsing Error] Unsupported type for '{}'",
            path
        )),
    }
}
