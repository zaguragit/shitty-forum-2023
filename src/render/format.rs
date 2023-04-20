
use chrono::{Utc, DateTime};

pub fn format_date_time(datetime: &DateTime<Utc>) -> String {
    let d = Utc::now().signed_duration_since(datetime.clone());
    if d.num_days() > 365 {
        datetime.format("%b %d %Y").to_string()
    } else if d.num_days() > 0 {
        datetime.format("%b %d").to_string()
    } else {
        format!("{}h ago", d.num_hours())
    }
}