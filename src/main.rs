use chrono::{Datelike, Utc};

mod formatter;

fn main() {
    let current_year:u16 = Utc::now().year() as u16;
    println!("The current year is {}", current_year);
}
