use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct NewTwitchEventMessage {
    pub event: TwitchEvent,
    pub message_id: String,
    pub message_at: String,
}

#[derive(Debug)]
pub enum TwitchEvent {
    ChannelFollow(FollowEvent),
    ChannelSubscribe(SubscribeEvent),
    ChannelRaid(RaidEvent),
    ChannelSubGift(ChannelGiftMessage),
}

#[derive(Debug)]
pub struct FollowEvent {
    pub user_name: String,
    pub user_id: i64,
}

#[derive(Debug)]
pub struct RaidEvent {
    pub from_broadcaster_user_id: String,
    pub from_broadcaster_user_login: String,
    pub from_broadcaster_user_name: String,
    pub to_broadcaster_user_id: String,
    pub to_broadcaster_user_login: String,
    pub to_broadcaster_user_name: String,
    pub viewers: i64,
}

#[derive(Debug)]
pub struct SubscribeEvent {
    pub broadcaster_user_id: i64,
    pub broadcaster_user_name: String,
    pub user_name: String,
    pub user_id: i64,
    pub is_gift: bool,
    pub tier: NullSubTier,
}

// Path: messages/src/lib.rs
// { "{ "message": "hello world", "image_url": "https:://something.com" }" }
#[derive(Serialize, Deserialize, Debug)]
pub struct DisplayMessage {
    pub message: String,
    pub image_url: String,
    pub sound_url: String,
    pub display_time: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelGiftMessage {
     /// The broadcaster user ID.
     pub broadcaster_user_id: String,
     /// The broadcaster login.
     pub broadcaster_user_login: String,
     /// The broadcaster display name.
     pub broadcaster_user_name: String,
     /// The number of subscriptions gifted by this user in the channel. This value is null for anonymous gifts or if the gifter has opted out of sharing this information.
     pub cumulative_total: Option<i64>,
     /// Whether the subscription gift was anonymous.
     pub is_anonymous: bool,
     /// The tier of subscriptions in the subscription gift.
     pub tier: NullSubTier,
     /// The number of subscriptions in the subscription gift.
     pub total: i64,
     /// The user ID of the user who sent the subscription gift. Set to null if it was an anonymous subscription gift.
     pub user_id: Option<String>,
     /// The user login of the user who sent the gift. Set to null if it was an anonymous subscription gift.
     pub user_login: Option<String>,
     /// The user display name of the user who sent the gift. Set to null if it was an anonymous subscription gift.
     pub user_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NullSubTier {
    /// Tier 1. $4.99
    Tier1(String),
    /// Tier 1. $9.99
    Tier2(String),
    /// Tier 1. $24.99
    Tier3(String),
    /// Prime subscription
    Prime(String),
    /// Other
    Other(String),
}