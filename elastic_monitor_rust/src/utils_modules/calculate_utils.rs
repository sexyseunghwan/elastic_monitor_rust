use crate::common::*;

pub fn get_decimal_round_conversion(value: f64, decimal: i32) -> Result<f64, anyhow::Error> {
    let square_num = decimal as f64 * 10.0;

    if square_num == 0.0 {
        return Err(anyhow!(
            "[Error][get_decimal_round_conversion()] Dividend cannot be zero"
        ));
    }

    Ok((value * square_num).round() / square_num)
}

pub fn get_percentage_transformation(divisor: i64, dividend: i64) -> Result<f64, anyhow::Error> {
    let divisor_f64: f64 = divisor as f64;
    let dividend_f64: f64 = dividend as f64;

    if dividend_f64 == 0.0 {
        warn!("[Warn][get_percentage_transformation()] Dividend cannot be zero");
        return Ok(0.0);
    }

    Ok((divisor_f64 / dividend_f64) * 100.0)
}

pub fn get_percentage_round_conversion(
    divisor: i64,
    dividend: i64,
    decimal: i32,
) -> Result<f64, anyhow::Error> {
    let percentage: f64 = get_percentage_transformation(divisor, dividend)?;
    let round_conversion: f64 = get_decimal_round_conversion(percentage, decimal)?;
    Ok(round_conversion)
}
