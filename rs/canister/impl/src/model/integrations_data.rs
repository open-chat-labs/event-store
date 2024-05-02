use event_store_canister::IndexedEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct IntegrationsData {
    #[cfg(feature = "dapp-radar")]
    pub dapp_radar: crate::integrations::dapp_radar::DappRadarData,
}

#[allow(unused_variables)]
impl IntegrationsData {
    pub fn push_event(&mut self, event: IndexedEvent) {
        #[cfg(feature = "dapp-radar")]
        if let Some(user) = event.user {
            self.dapp_radar
                .push_event(event.index, user, event.timestamp);
        }
    }
}
