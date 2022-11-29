use std::str::FromStr;

struct Time {
    hour: u16,
    minute: u16,
}

impl FromStr for Time {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hour, minute) = s.split_once(':').ok_or("should have a colon")?;

        let hour = hour.parse().map_err(|_| "hour should be a number")?;
        let minute = minute.parse().map_err(|_| "minute should be a number")?;

        if hour > 23 {
            return Err("hour should be less than 24");
        }

        if minute > 59 {
            return Err("minute should be less than 60");
        }

        Ok(Time { hour, minute })
    }
}

#[fncli::cli]
fn main(
    Time { hour, minute }: Time,
    Time {
        hour: hour2,
        minute: minute2,
    }: Time,
) {
    let elapsed = i32::from(hour2 * 60 + minute2) - i32::from(hour * 60 + minute);

    if elapsed < 0 {
        print!("time traveler detected: ");
    }

    let hours = elapsed / 60;
    let minutes = elapsed % 60;

    println!("{hours} hours and {minutes} minutes have elapsed from {hour}:{minute} to {hour2}:{minute2}");
}
