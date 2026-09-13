use crate::{Error, Health, Item, ItemInput, ItemView, Result, Settings};
use chrono::{DateTime, Days, Duration, LocalResult, Months, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

pub fn date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| Error::Validation("invalid_date"))
        .and_then(|d| {
            if ("1900-01-01"..="2200-12-31").contains(&value) {
                Ok(d)
            } else {
                Err(Error::Validation("invalid_date"))
            }
        })
}

pub fn zone(settings: &Settings) -> Result<Tz> {
    settings
        .timezone
        .parse()
        .map_err(|_| Error::Validation("invalid_timezone"))
}

pub fn time(value: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| Error::Validation("invalid_time"))
}

pub fn next_date(from: NaiveDate, repeat: &str, every: u32) -> Result<NaiveDate> {
    if every == 0 || every > 999 {
        return Err(Error::Validation("invalid_interval"));
    }
    let result = match repeat {
        "days" => from.checked_add_days(Days::new(every.into())),
        "weeks" => from.checked_add_days(Days::new(u64::from(every) * 7)),
        "months" => from.checked_add_months(Months::new(every)),
        "years" => from.checked_add_months(Months::new(every * 12)),
        _ => return Err(Error::Validation("invalid_interval")),
    }
    .ok_or(Error::Validation("invalid_date"))?;
    date(&result.to_string())
}

pub fn validate_input(input: &ItemInput) -> Result<()> {
    if input.name.trim().is_empty() || input.name.chars().count() > 120 {
        return Err(Error::Validation("invalid_name"));
    }
    if input.notes.chars().count() > 4000 {
        return Err(Error::Validation("invalid_notes"));
    }
    validate_appearance(&input.icon, &input.color)?;
    let start = date(&input.started_on)?;
    let due = date(&input.due_on)?;
    if due < start {
        return Err(Error::Validation("date_order"));
    }
    if !["once", "days", "weeks", "months", "years"].contains(&input.repeat.as_str())
        || input.every == 0
        || input.every > 999
        || input.advance_days > 90
    {
        return Err(Error::Validation("invalid_interval"));
    }
    Ok(())
}

pub fn validate_appearance(icon: &str, color: &str) -> Result<()> {
    if icon.is_empty()
        || icon.len() > 48
        || !icon.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(Error::Validation("invalid_icon"));
    }
    if ![
        "sage", "peach", "lavender", "blue", "yellow", "rose", "mint", "teal", "coral", "plum",
        "sand", "slate",
    ]
    .contains(&color)
    {
        return Err(Error::Validation("invalid_color"));
    }
    Ok(())
}

pub fn validate_settings(s: &Settings) -> Result<()> {
    zone(s)?;
    time(&s.reminder_time)?;
    time(&s.quiet_start)?;
    time(&s.quiet_end)?;
    if !["ru", "en"].contains(&s.language.as_str())
        || !["light", "dark", "system"].contains(&s.theme.as_str())
        || !["plant", "cat", "frog"].contains(&s.pet.as_str())
        || s.home_name.chars().count() > 60
    {
        return Err(Error::Validation("invalid_settings"));
    }
    Ok(())
}

/// Resolve a civil time deterministically: first occurrence in a fold, first
/// representable minute after a gap. This also covers skipped civil days.
pub fn resolve(day: NaiveDate, at: NaiveTime, tz: Tz) -> Result<DateTime<Utc>> {
    let local = day.and_time(at);
    for offset in 0..=2880 {
        match tz.from_local_datetime(&(local + Duration::minutes(offset))) {
            LocalResult::Single(d) => return Ok(d.with_timezone(&Utc)),
            LocalResult::Ambiguous(a, b) => return Ok(a.min(b).with_timezone(&Utc)),
            LocalResult::None => (),
        }
    }
    Err(Error::Validation("invalid_time"))
}

pub fn snooze(now: DateTime<Utc>, choice: &str, settings: &Settings) -> Result<DateTime<Utc>> {
    let tz = zone(settings)?;
    let today = now.with_timezone(&tz).date_naive();
    match choice {
        "hour" => Ok(now + Duration::hours(1)),
        "evening" => {
            let tonight = resolve(today, time("19:00")?, tz)?;
            if tonight > now {
                Ok(tonight)
            } else {
                resolve(today + Days::new(1), time("19:00")?, tz)
            }
        }
        "tomorrow" => resolve(today + Days::new(1), time(&settings.reminder_time)?, tz),
        _ => Err(Error::Validation("invalid_snooze")),
    }
}

pub fn view(item: Item, today: NaiveDate) -> Result<ItemView> {
    let days_left = (date(&item.input.due_on)? - today).num_days() as i32;
    let total = (date(&item.input.due_on)? - date(&item.input.started_on)?)
        .num_days()
        .max(1);
    let soon_window = ((total as f64 * 0.15).ceil() as i32).clamp(1, 7);
    let status = if item.completed {
        "completed"
    } else if days_left < 0 {
        "overdue"
    } else if days_left == 0 {
        "today"
    } else if days_left <= soon_window {
        "soon"
    } else {
        "good"
    };
    Ok(ItemView {
        item,
        status: status.into(),
        days_left,
        remaining: (days_left as f64 / total as f64).clamp(0.0, 1.0),
    })
}

pub fn health(items: &[ItemView], completions: u32, timely_recent: u32) -> Health {
    let active: Vec<_> = items.iter().filter(|v| !v.item.completed).collect();
    let pressure: f64 = active
        .iter()
        .map(|v| match v.status.as_str() {
            "overdue" => 0.35 + 0.65 * (f64::from(-v.days_left) / 14.0).min(1.0),
            "today" => 0.15,
            "soon" => 0.04,
            _ => 0.0,
        })
        .sum();
    let score = (100.0 - 80.0 * pressure / (active.len() as f64 + 4.0)
        + f64::from(timely_recent.min(5)))
    .round()
    .clamp(0.0, 100.0) as u32;
    Health {
        score,
        mood: if score >= 90 {
            "happy"
        } else if score >= 75 {
            "calm"
        } else if score >= 55 {
            "sleepy"
        } else {
            "care"
        }
        .into(),
        completions,
        level: completions / 10 + 1,
        level_progress: completions % 10,
        overdue: active.iter().filter(|v| v.status == "overdue").count() as u32,
        today: active.iter().filter(|v| v.status == "today").count() as u32,
        soon: active.iter().filter(|v| v.status == "soon").count() as u32,
    }
}

pub fn quiet_until(now: DateTime<Utc>, settings: &Settings) -> Result<Option<DateTime<Utc>>> {
    let tz = zone(settings)?;
    let local = now.with_timezone(&tz);
    let start = time(&settings.quiet_start)?;
    let end = time(&settings.quiet_end)?;
    if start == end {
        return Ok(None);
    }
    let at = local.time();
    let inside = if start > end {
        at >= start || at < end
    } else {
        at >= start && at < end
    };
    if !inside {
        return Ok(None);
    }
    let day = local.date_naive() + Days::new(u64::from(start > end && at >= start));
    Ok(Some(resolve(day, end, tz)?))
}
