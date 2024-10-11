use crate::common::*;


/*
    
*/
pub fn get_current_utc_naivedate() -> NaiveDate {
    
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.date_naive()
}

pub fn get_current_utc_naivedate_str(fmt: &str) -> String {

    let curr_time = get_current_utc_naivedate();
    get_str_from_naivedate(curr_time, fmt)
    
}

/*
    Function that converts the date data 'naivedate' format to the string format
*/
pub fn get_str_from_naivedate(naive_date: NaiveDate, fmt: &str) -> String {
    //naive_date.format("%Y%m%d").to_string()
    naive_date.format(fmt).to_string()
}


/*
    Function that converts the date data 'naivedatetime' format to String format
*/
pub fn get_str_from_naive_datetime(naive_datetime: NaiveDateTime) -> String {
    
    naive_datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()

}


/*
    Function to change 'string' data format to 'NaiveDateTime' format
*/
pub fn get_naive_datetime_from_str(date: &str, format: &str) -> Result<NaiveDateTime, anyhow::Error> {
    
    NaiveDateTime::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_datetime_from_str()] Failed to parse date string: {:?} : {:?}", date, e)) 
}


/*
    Function to change 'string' data format to 'NaiveDate' format
*/
pub fn get_naive_date_from_str(date: &str, format: &str) -> Result<NaiveDate, anyhow::Error> {
    
    NaiveDate::parse_from_str(date, format)
        .map_err(|e| anyhow!("[Datetime Parsing Error][get_naive_date_from_str()] Failed to parse date string: {:?} : {:?}", date, e))
    
}
