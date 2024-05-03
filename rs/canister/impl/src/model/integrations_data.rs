use event_store_canister::IndexedEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct IntegrationsData {
    #[cfg(feature = "dapp-radar")]
    pub dapp_radar: crate::integrations::dapp_radar::DappRadarData,
}

#[allow(unused_mut)]
#[allow(unused_variables)]
impl IntegrationsData {
    pub fn push_event(&mut self, event: IndexedEvent) {
        #[cfg(feature = "dapp-radar")]
        if let Some(user) = event.user {
            self.dapp_radar
                .push_event(event.index, user, event.timestamp);
        }
    }

    pub fn next_event_index(&self) -> Option<u64> {
        let mut index = u64::MAX;

        #[cfg(feature = "dapp-radar")]
        {
            index = std::cmp::min(index, self.dapp_radar.next_event_index());
        }

        if index == u64::MAX {
            None
        } else {
            Some(index)
        }
    }
}
