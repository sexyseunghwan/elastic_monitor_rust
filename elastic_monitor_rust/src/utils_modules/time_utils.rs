use crate::common::*;


#[doc = "시스템에 호환되도록 날짜 타입을 변환해주는 함수"]
pub fn format_datetime(dt: NaiveDateTime) -> Result<String, anyhow::Error> {
    get_str_from_naivedatetime(dt, "%Y-%m-%dT%H:%M:%SZ")
}

#[doc = "Functions that return the current UTC time -> NaiveDatetime"]
pub fn get_currnet_utc_naivedatetime() -> NaiveDateTime {
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.naive_local()
}


#[doc = "Function that converts the date data 'naivedate' format to the string format"]
pub fn get_str_from_naivedatetime(
    naive_date: NaiveDateTime,
    fmt: &str,
) -> Result<String, anyhow::Error> {
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}

#[doc = "현재 UTC 시각 기준으로 지정된 초(sec_space) 이전 시각까지의 시간 범위를 생성."]
/// # Arguments
/// * `sec_space` - 현재 시각으로부터 과거로 몇 초를 뺄 것인지 지정한다.
/// 
/// # Returns
/// `Ok((now, past, now_str, past_str))` 형태의 튜플을 반환:
/// - `now`: 현재 UTC 시간 (`NaiveDateTime`)
/// - `past`: 현재로부터 `sec_space`초 전의 UTC 시간 (`NaiveDateTime`)
/// - `now_str`: 현재 UTC 시간을 문자열로 포맷한 값 (`String`)
/// - `past_str`: 과거 UTC 시간을 문자열로 포맷한 값 (`String`)
pub fn make_time_range(sec_space: i64) -> Result<(NaiveDateTime, NaiveDateTime, String, String), anyhow::Error> {
    let now: NaiveDateTime = get_currnet_utc_naivedatetime();
    let past: NaiveDateTime = now - chrono::Duration::seconds(sec_space); //20

    let now_str: String = format_datetime(now)?;
    let past_str: String = format_datetime(past)?;

    Ok((now, past, now_str, past_str))
}