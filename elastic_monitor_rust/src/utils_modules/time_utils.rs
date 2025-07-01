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