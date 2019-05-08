use crate::Event;
use crate::event::event_hash::EventHash;
use crate::PeerId;
use crate::FlagTable;
use failure::Error;

// Store trait for Poset
pub trait Store {
    fn topological_events(&self, start, finish: u64) -> Result<Vec<Event>, Error>;
    fn set_event(&self, ev: Event) -> Result<(),Error>;
    fn get_event(&self, hash: EventHash) -> Result<Event, Error>;
    fn get_store_path(&self) -> Result<String, Error>;
    fn get_clotho_check(&self, frame: u64, hash: EventHash) -> Result<EventHash, Error>;
    fn get_clotho_creator_check(&self, frame: u64, ctreator: PeerId) -> Result<EventHash, Error>;
    fn add_clotho_check(&self, frame: u64, creator: PeerID, hash: EventHash) -> Result<(), Error>;
    fn add_time_table(&self, to: EventHash, from: EventHash, lamport_time: u64) -> Result<(), Error>;
    fn get_time_table(&self, hash: EventHash) -> Result<FlagTable, Error>;
    fn check_frame_finality(&self, frame: u64) -> bool;
    // process_out_frame() uses for debug/tracking/research purposes to dump frame when it's finalised
    fn process_out_frame(&self, frame: u64, address: String) ->Result<(), Error>;
}