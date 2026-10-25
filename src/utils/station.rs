use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;

use crate::error::AppError;

pub fn get_today_date(timezone: Tz) -> NaiveDate {
    Utc::now().with_timezone(&timezone).date_naive()
}

pub fn get_current_time(timezone: Tz) -> NaiveTime {
    Utc::now().with_timezone(&timezone).time()
}

pub fn generate_days(days: u8, timezone: Tz) -> Vec<NaiveDate> {
    let today = get_today_date(timezone);

    let mut days_vec: Vec<NaiveDate> = vec![];
    for i in 0..days {
        let duration = Duration::days(i.into());
        let current_day = today + duration;
        days_vec.push(current_day);
    }

    days_vec
}

pub fn check_if_date_is_available(
    date: NaiveDate,
    timezone: Tz,
    max_booking_days: u8,
) -> Result<(), AppError> {
    let current_date = get_today_date(timezone);
    let max_date = current_date + Duration::days((max_booking_days - 1).into());
    if date > max_date || date < current_date {
        return Err(AppError::BadRequest(format!(
            "Date must be between {} and {}",
            current_date, max_date
        )));
    }

    Ok(())
}
