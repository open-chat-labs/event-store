use event_store_canister::TimestampMillis;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const HOURLY_MAX_ENTRIES: usize = 24 * 70;
const PAGE_SIZE: usize = 1000;

#[derive(Serialize, Deserialize, Default)]
pub struct DappRadarData {
    daily: BTreeMap<(u32, u8, u8), EventsPerUser>,
    hourly: BTreeMap<(u32, u8, u8, u8), EventsPerUser>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct EventsPerUser {
    per_user: BTreeMap<String, u32>,
}

#[derive(Serialize)]
pub struct DappRadarResponse {
    results: Vec<DappRadarResponseEntry>,
    #[serde(rename = "pageCount")]
    page_count: u32,
}

#[derive(Serialize)]
struct DappRadarResponseEntry {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    user: String,
    transactions: u32,
}

impl DappRadarData {
    pub fn push_event(&mut self, user: String, timestamp: TimestampMillis) {
        let datetime =
            time::OffsetDateTime::from_unix_timestamp((timestamp / 1000) as i64).unwrap();

        let year = datetime.year() as u32;
        let month = datetime.month() as u8;
        let day = datetime.day();
        let hour = datetime.hour();

        let day_key = (year, month, day);
        let hour_key = (year, month, day, hour);

        self.daily.entry(day_key).or_default().push(user.clone());
        self.hourly.entry(hour_key).or_default().push(user);

        while self.hourly.len() > HOURLY_MAX_ENTRIES {
            self.hourly.pop_first();
        }
    }

    pub fn hourly(&self, year: u32, month: u8, day: u8, page: usize) -> DappRadarResponse {
        let all_results: Vec<_> = self
            .hourly
            .range(&(year, month, day, 0)..&(year, month, day + 1, 0))
            .flat_map(|((_, _, _, hour), events)| {
                events
                    .per_user
                    .iter()
                    .map(move |(user, count)| DappRadarResponseEntry {
                        date_time: Some(format!("{year}-{month:02}-{day:02} {hour:02}:00:00")),
                        user: user.clone(),
                        transactions: *count,
                    })
            })
            .collect();

        Self::extract_page(all_results, page)
    }

    pub fn daily(&self, year: u32, month: u8, day: u8, page: usize) -> DappRadarResponse {
        let all_results: Vec<_> = self
            .daily
            .range(&(year, month, day)..&(year, month, day + 1))
            .flat_map(|((_, _, _), events)| {
                events
                    .per_user
                    .iter()
                    .map(|(user, count)| DappRadarResponseEntry {
                        date_time: None,
                        user: user.clone(),
                        transactions: *count,
                    })
            })
            .collect();

        Self::extract_page(all_results, page)
    }

    fn extract_page(all_results: Vec<DappRadarResponseEntry>, page: usize) -> DappRadarResponse {
        if all_results.is_empty() {
            DappRadarResponse {
                results: Vec::new(),
                page_count: 0,
            }
        } else {
            let page_count = (((all_results.len() - 1) / PAGE_SIZE) + 1) as u32;

            DappRadarResponse {
                results: all_results
                    .into_iter()
                    .skip(page * PAGE_SIZE)
                    .take(PAGE_SIZE)
                    .collect(),
                page_count,
            }
        }
    }
}

impl EventsPerUser {
    fn push(&mut self, user: String) {
        *self.per_user.entry(user).or_default() += 1;
    }
}
