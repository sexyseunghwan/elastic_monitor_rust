use crate::common::*;



/*  
    Functions that return the current UTC time -> NaiveDate
*/
pub fn get_current_utc_naivedate() -> NaiveDate {
    
    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.date_naive()
}

/*
    Functions that return the current UTC time -> NaiveDatetime
*/
pub fn get_currnet_utc_naivedatetime() -> NaiveDateTime {

    let utc_now: DateTime<Utc> = Utc::now();
    utc_now.naive_local()
}


/*  
    Function that returns the current UTC time as a string
*/
// pub fn get_current_utc_naivedate_str(fmt: &str) -> String {

//     let curr_time = get_current_utc_naivedate();
//     get_str_from_naivedate(curr_time, fmt)
    
// }

/*
    Function that converts the date data 'naivedate' format to the string format
*/
pub fn get_str_from_naivedatetime(naive_date: NaiveDateTime, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}


/*
    Function that converts the date data 'naivedate' format to the string format
*/
pub fn get_str_from_naivedate(naive_date: NaiveDate, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_date.format(fmt).to_string();
    Ok(result_date)
}


/*
    Function that converts the date data 'naivedatetime' format to String format
*/
pub fn get_str_from_naive_datetime(naive_datetime: NaiveDateTime, fmt: &str) -> Result<String, anyhow::Error> {
    
    let result_date = naive_datetime.format(fmt).to_string();
    Ok(result_date)
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
