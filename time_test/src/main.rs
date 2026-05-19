use time::{PrimitiveDateTime, Date, Time, Month};
fn main() {
    let dt = PrimitiveDateTime::new(
        Date::from_calendar_date(2026, Month::May, 14).unwrap(),
        Time::from_hms_milli(9, 22, 32, 105).unwrap()
    );
    println!("{}", serde_json::to_string(&dt).unwrap());
}
