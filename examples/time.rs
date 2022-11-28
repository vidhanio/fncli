use std::str::FromStr;
struct Time {
    hour: u8,
    minute: u8,
}

impl FromStr for Time {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hour, minute) = s.split_once(':').ok_or("should have a colon")?;
        let hour = hour.parse().map_err(|_| "invalid hour")?;
        let minute = minute.parse().map_err(|_| "invalid minute")?;
        Ok(Time { hour, minute })
    }
}

#[fncli::cli]
fn main(Time { hour, minute }: Time) {
    println!("{} hours, {} minutes", hour, minute);
}
