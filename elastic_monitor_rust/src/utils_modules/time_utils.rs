use crate::common::*;

#[doc = "Generates a time range starting at (current UTC time − sec_space seconds) and ending at the current UTC time."]
pub fn make_time_range(sec_space: i64) -> (DateTime<Utc>, DateTime<Utc>, String, String) {
    let now: DateTime<Utc> = Utc::now();
    let past: DateTime<Utc> = now - chrono::Duration::seconds(sec_space);

    let now_str: String = convert_date_to_str_full(now, Utc);
    let past_str: String = convert_date_to_str_full(past, Utc);

    (now, past, now_str, past_str)
}

#[doc = "Standard Function of Datetime"]
fn convert_date_to_str<Tz>(
    time: DateTime<Tz>,
    tz: Tz, // Timezone (Utc, Local, FixedOffset ...)
    format: &str,
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    time.with_timezone(&tz).format(format).to_string()
}

pub fn convert_date_to_str_full<Tz>(
    time: DateTime<Tz>,
    tz: Tz, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y-%m-%dT%H:%M:%SZ")
}

pub fn convert_date_to_str_ymd_mail<Tz>(time: DateTime<Tz>, tz: Tz) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y.%m.%d")
}

pub fn convert_date_to_str_ymd<Tz>(
    time: DateTime<Tz>,
    tz: Tz, // Timezone (Utc, Local, FixedOffset ...)
) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y%m%d")
}

pub fn convert_date_to_str_ymdhms<Tz>(time: DateTime<Tz>, tz: Tz) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y%m%d%H%M%S")
}

pub fn convert_date_to_str_human<Tz>(time: DateTime<Tz>, tz: Tz) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    convert_date_to_str(time, tz, "%Y.%m.%d %H:%M:%S")
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
