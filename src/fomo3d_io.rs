use codec::{Decode, Encode};
use gstd::{ActorId, TypeInfo};

#[derive(Debug, Decode, Encode, Clone, TypeInfo)]
pub struct Fomo3dInit {
    pub token_address: ActorId,
}

#[derive(Debug, Decode, Encode, Clone, TypeInfo)]
pub enum Fomo3dAction {
    BuyKey,
}

#[derive(Debug, Decode, Encode, Clone, TypeInfo)]
pub enum Fomo3dEvent {
    KeyBought { price: u128, address: ActorId },
}
