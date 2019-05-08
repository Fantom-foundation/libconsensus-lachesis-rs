use std::collections::HashMap;
use crate::event::event_hash::EventHash;

// FlagTable is a map from EventHash into Frame number (u64)
pub type FlagTable = HashMap<EventHash, u64>;
