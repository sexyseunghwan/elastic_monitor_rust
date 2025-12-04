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
pub fn make_time_range(
    sec_space: i64,
) -> Result<(NaiveDateTime, NaiveDateTime, String, String), anyhow::Error> {
    let now: NaiveDateTime = get_currnet_utc_naivedatetime();
    let past: NaiveDateTime = now - chrono::Duration::seconds(sec_space); //20

    let now_str: String = format_datetime(now)?;
    let past_str: String = format_datetime(past)?;

    Ok((now, past, now_str, past_str))
}

fn convert_date_to_str<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // Timezone (Utc, Local, FixedOffset ...)
    format: &str,
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    time.with_timezone(&tz).format(format).to_string()
}

pub fn convert_date_to_str_full<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y-%m-%dT%H:%M:%SZ")
}

pub fn convert_date_to_str_ymd<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y%m%d")
}

pub fn convert_date_to_str_ymdhms<Tz, TzOut>(
    time: DateTime<Tz>,
    tz: TzOut, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
    TzOut: TimeZone,
    TzOut::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y%m%d%H%M%S")
}

pub fn convert_str_to_datetime<Tz>(time: &str, tz: Tz) -> anyhow::Result<DateTime<Tz>>
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    let dt_datetime: DateTime<Tz> =
        DateTime::parse_from_rfc3339(time)
        .context("[time_utils.rs -> convert_str_to_datetime] An error occurred during the conversion of time information.")?
        .with_timezone(&tz);

    Ok(dt_datetime)
}

#[doc = "Convert UTC DateTime to Local DateTime"]
/// # Arguments
/// * `date_at_str` - UTC DateTime to convert
///
/// # Returns
/// * `Ok(DateTime<Local>)` - Converted Local DateTime
pub fn convert_utc_to_local(date_at_str: &str) -> anyhow::Result<DateTime<Local>> {
    let utc_time: DateTime<Utc> =
        convert_str_to_datetime(date_at_str, Utc).context("[convert_utc_to_local] error")?;
    let local_time: DateTime<Local> = utc_time.with_timezone(&Local);
    Ok(local_time)
}
