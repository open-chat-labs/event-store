use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry::Vacant;
use std::collections::BTreeMap;

const DEFAULT_WINDOW_DURATION: Milliseconds = 60 * 60 * 1000; // 1 hour

type Milliseconds = u64;
type TimestampMillis = u64;

#[derive(Serialize, Deserialize)]
pub struct EventDeduper {
    window_duration: Milliseconds,
    recently_added: BTreeMap<u128, TimestampMillis>,
    recently_added_last_pruned: TimestampMillis,
}

impl EventDeduper {
    pub fn new(window_duration: Milliseconds) -> EventDeduper {
        EventDeduper {
            window_duration,
            recently_added: BTreeMap::new(),
            recently_added_last_pruned: 0,
        }
    }

    pub fn try_push(&mut self, key: u128, now: TimestampMillis) -> bool {
        self.prune_if_due(now);

        match self.recently_added.entry(key) {
            Vacant(e) => {
                e.insert(now);
                true
            }
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.recently_added.is_empty()
    }

    pub fn len(&self) -> usize {
        self.recently_added.len()
    }

    fn prune_if_due(&mut self, now: TimestampMillis) {
        if now.saturating_sub(self.recently_added_last_pruned) > self.window_duration / 2 {
            let cutoff = now.saturating_sub(self.window_duration);
            self.recently_added.retain(|_, ts| *ts > cutoff);
            self.recently_added_last_pruned = now;
        }
    }
}

impl Default for EventDeduper {
    fn default() -> Self {
        EventDeduper::new(DEFAULT_WINDOW_DURATION)
    }
}
