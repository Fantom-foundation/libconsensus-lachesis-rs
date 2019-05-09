use crate::event::event_hash::EventHash;
use std::collections::HashMap;

// FlagTable is a map from EventHash into Frame number (u64)
pub type FlagTable = HashMap<EventHash, u64>;
