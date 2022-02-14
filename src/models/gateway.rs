use super::*;

bitflags::bitflags! {
    pub struct Intent: u32 {
        /// - PARTY_CREATE
        /// - PARTY_UPDATE
        /// - PARTY_DELETE
        /// - PARTY_ROLE_CREATE
        /// - PARTY_ROLE_UPDATE
        /// - PARTY_ROLE_DELETE
        /// - CHANNEL_CREATE
        /// - CHANNEL_UPDATE
        /// - CHANNEL_DELETE
        /// - CHANNEL_PINS_UPDATE
        const PARTIES                   = 1 << 0;

        /// - PARTY_MEMBER_ADD
        /// - PARTY_MEMBER_UPDATE
        /// - PARTY_MEMBER_REMOVE
        const PARTY_MEMBERS             = 1 << 1;

        /// - PARTY_BAN_ADD
        /// - PARTY_BAN_REMOVE
        const PARTY_BANS                = 1 << 2;

        /// - PARTY_EMOJIS_UPDATE
        const PARTY_EMOTES              = 1 << 3;

        /// - PARTY_INTEGRATIONS_UPDATE
        /// - INTEGRATION_CREATE
        /// - INTEGRATION_UPDATE
        /// - INTEGRATION_DELETE
        const PARTY_INTEGRATIONS        = 1 << 4;

        /// - WEBHOOKS_UPDATE
        const PARTY_WEBHOOKS            = 1 << 5;

        /// - INVITE_CREATE
        /// - INVITE_DELETE
        const PARTY_INVITES             = 1 << 6;

        /// - VOICE_STATE_UPDATE
        const VOICE_STATUS              = 1 << 7;

        /// - PRESENCE_UPDATE
        const PRESENCE                  = 1 << 8;

        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - MESSAGE_DELETE_BULK
        const MESSAGES                  = 1 << 9;

        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOTE
        const MESSAGE_REACTIONS         = 1 << 10;

        /// - TYPING_START
        const MESSAGE_TYPING            = 1 << 11;

        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - CHANNEL_PINS_UPDATE
        const DIRECT_MESSAGES           = 1 << 12;

        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOTE
        const DIRECT_MESSAGE_REACTIONS  = 1 << 13;

        /// - TYPING_START
        const DIRECT_MESSAGE_TYPING     = 1 << 14;
    }
}

serde_shims::bitflags::impl_serde_for_bitflags!(Intent);

pub mod commands {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Identify {
        pub auth: AuthToken,
        pub intent: Intent,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SetPresence {
        #[serde(flatten)]
        pub presence: UserPresence,
    }
}

pub mod events {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Hello {
        /// Number of milliseconds between heartbeats
        pub heartbeat_interval: u32,
    }

    impl Default for Hello {
        fn default() -> Self {
            Hello {
                heartbeat_interval: 45000, // 45 seconds
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Ready {
        pub user: User,
        pub dms: Vec<Room>,
        pub parties: Vec<Party>,
        pub session: Snowflake,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TypingStart {
        pub room: Snowflake,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub party: Option<Snowflake>,
        pub user: Snowflake,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub member: Option<PartyMember>,
        // maybe timestamp?
        //ts: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PartyPositionUpdate {
        pub id: Snowflake,
        pub position: i16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserPresenceEvent {
        pub user: User,
        pub presence: UserPresence,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MessageDeleteEvent {
        pub id: Snowflake,
        pub room_id: Snowflake,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub party_id: Option<Snowflake>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RoleDeleteEvent {
        pub id: Snowflake,
        pub party_id: Snowflake,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RoomDeleteEvent {
        pub id: Snowflake,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub party_id: Option<Snowflake>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PartyMemberEvent {
        pub party_id: Snowflake,

        #[serde(flatten)]
        pub member: PartyMember,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum PartyUpdateEvent {
        Position(PartyPositionUpdate),
        Full(Party),
    }

    //#[derive(Debug, Clone, Serialize, Deserialize)]
    //pub struct PresenceUpdate {
    //    pub user_id: Snowflake,
    //    pub presence: UserPresence,
    //}
}

pub mod message {
    use super::Snowflake;

    use serde_repr::{Deserialize_repr, Serialize_repr};

    use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};

    pub use client::Message as ClientMsg;
    pub use server::Message as ServerMsg;

    #[inline]
    fn is_default<T>(value: &T) -> bool
    where
        T: Default + Eq,
    {
        *value == T::default()
    }

    macro_rules! decl_msgs {
        ($($code:expr => $opcode:ident $(:$Default:ident)? {
            $( $(#[$field_meta:meta])* $field:ident : $ty:ty),*$(,)?
        }),*$(,)?) => {paste::paste!{
            #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
            #[repr(u8)]
            pub enum Opcode {
                $($opcode = $code,)*
            }

            pub mod payloads { use super::*; $(
                #[derive(Debug, Serialize, Deserialize)]
                $(#[derive($Default, PartialEq, Eq)])?
                pub struct [<$opcode Payload>] {
                    $($(#[$field_meta])* pub $field : $ty,)*
                }
            )*}

            #[derive(Debug, Serialize)]
            #[serde(untagged)] // custom tagging
            pub enum Message {$(
                $opcode {
                    #[serde(rename = "o")]
                    op: Opcode,

                    #[serde(rename = "p")]
                    $(#[serde(skip_serializing_if = "" [< is_ $Default:lower >] "" )])?
                    payload: payloads::[<$opcode Payload>],
                },)*
            }

            impl Message {
                $(
                    #[inline]
                    pub const fn [<$opcode:lower>](payload: payloads::[<$opcode Payload>]) -> Message {
                        Message::$opcode { op: Opcode::$opcode, payload }
                    }

                    #[inline]
                    pub fn [<new_ $opcode:lower>]($($field: impl Into<$ty>),*) -> Message {
                        Message::$opcode { op: Opcode::$opcode, payload: payloads::[<$opcode Payload>] { $($field: $field.into()),* }}
                    }
                )*
            }

            impl<'de> Deserialize<'de> for Message {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>
                {
                    use std::fmt;

                    #[derive(Deserialize)]
                    enum Field {
                        #[serde(rename = "o")]
                        Opcode,

                        #[serde(rename = "p")]
                        Payload,
                    }

                    struct MessageVisitor;

                    impl<'de> Visitor<'de> for MessageVisitor {
                        type Value = Message;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("struct Message")
                        }

                        fn visit_map<V>(self, mut map: V) -> Result<Message, V::Error>
                        where
                            V: MapAccess<'de>,
                        {
                            let opcode = match map.next_entry()? {
                                Some((Field::Opcode, o)) => o,
                                _ => return Err(de::Error::custom("Missing opcode first")),
                            };

                            match opcode {
                                $(
                                    Opcode::$opcode => Ok(Message::$opcode {
                                        op: opcode,
                                        payload: match map.next_entry()? {
                                            Some((Field::Payload, payload)) => payload,
                                            $(None => $Default::default(),)?
                                            _ => return Err(de::Error::missing_field("payload")),
                                        }
                                    }),
                                )*
                                // _ => Err(de::Error::custom("Invalid opcode")),
                            }
                        }
                    }

                    deserializer.deserialize_struct("Message", &["o", "p"], MessageVisitor)
                }
            }
        }}
    }

    pub mod server {
        use super::*;

        use std::sync::Arc;

        use crate::models::{
            events::*, Intent, Message as RoomMessage, Party, PartyMember, Role, User, UserPresence,
        };

        type Room = (); // TODO

        // TODO: Check that this enum doesn't grow too large, allocate large payloads like Ready
        decl_msgs! {
            0 => Hello { #[serde(flatten)] inner: Hello },

            1 => HeartbeatACK: Default {},
            2 => Ready { #[serde(flatten)] inner: Box<Ready> },
            3 => InvalidSession: Default {},

            4 => PartyCreate { #[serde(flatten)] inner: Box<Party> },
            5 => PartyUpdate { #[serde(flatten)] inner: Box<PartyUpdateEvent> },
            6 => PartyDelete { id: Snowflake },

            7 => RoleCreate { #[serde(flatten)] inner: Box<Role> },
            8 => RoleUpdate { #[serde(flatten)] inner: Box<Role> },
            9 => RoleDelete { #[serde(flatten)] inner: Box<RoleDeleteEvent> },

            10 => MemberAdd     { #[serde(flatten)] inner: Box<PartyMemberEvent> },
            11 => MemberUpdate  { #[serde(flatten)] inner: Box<PartyMemberEvent> },
            12 => MemberRemove  { #[serde(flatten)] inner: Arc<PartyMemberEvent> },
            13 => MemberBan     { #[serde(flatten)] inner: Arc<PartyMemberEvent> },
            14 => MemberUnban   { #[serde(flatten)] inner: Box<PartyMemberEvent> },

            15 => RoomCreate { #[serde(flatten)] room: Box<Room> },
            16 => RoomUpdate { #[serde(flatten)] room: Box<Room> },
            17 => RoomDelete { #[serde(flatten)] room: Box<RoomDeleteEvent> },
            18 => RoomPinsUpdate {},

            19 => MessageCreate { #[serde(flatten)] msg: Box<RoomMessage> },
            20 => MessageUpdate { #[serde(flatten)] msg: Box<RoomMessage> },
            21 => MessageDelete { #[serde(flatten)] msg: Box<MessageDeleteEvent> },

            22 => MessageReactionAdd {},
            23 => MessageReactionRemove {},
            24 => MessageReactionRemoveAll {},
            25 => MessageReactionRemoveEmote {},

            26 => PresenceUpdate {
                party: Option<Snowflake>,
                #[serde(flatten)] inner: Arc<UserPresenceEvent>,
            },
            27 => TypingStart { #[serde(flatten)] t: Box<TypingStart> },
            28 => UserUpdate { user: Arc<User> }
        }

        impl Message {
            #[rustfmt::skip]
            pub fn matching_intent(&self) -> Option<Intent> {
                Some(match *self {
                    Message::PartyCreate { .. } |
                    Message::PartyDelete { .. } |
                    Message::PartyUpdate { .. } |
                    Message::RoleCreate { .. } |
                    Message::RoleDelete { .. } |
                    Message::RoleUpdate { .. } |
                    Message::RoomPinsUpdate { .. } |
                    Message::RoomCreate { .. } |
                    Message::RoomDelete { .. } |
                    Message::RoomUpdate { .. } => Intent::PARTIES,

                    Message::MemberAdd { .. } |
                    Message::MemberRemove { .. } |
                    Message::MemberUpdate { .. } => Intent::PARTY_MEMBERS,

                    Message::MemberBan {..} | Message::MemberUnban {..} => Intent::PARTY_BANS,

                    Message::MessageCreate { .. } |
                    Message::MessageDelete { .. } |
                    Message::MessageUpdate { .. } => Intent::MESSAGES,

                    Message::MessageReactionAdd { .. } |
                    Message::MessageReactionRemove { .. } |
                    Message::MessageReactionRemoveAll { .. } |
                    Message::MessageReactionRemoveEmote { .. } => Intent::MESSAGE_REACTIONS,

                    Message::PresenceUpdate { .. } => Intent::PRESENCE,
                    Message::TypingStart { .. } => Intent::MESSAGE_TYPING,

                    Message::Hello { .. } |
                    Message::HeartbeatACK { .. } |
                    Message::Ready { .. } |
                    Message::InvalidSession { .. } |
                    Message::UserUpdate { .. } => return None,
                })
            }
        }
    }

    pub mod client {
        use super::*;

        use crate::models::{
            commands::{Identify, SetPresence},
            Intent,
        };

        decl_msgs! {
            0 => Heartbeat: Default {},
            1 => Identify { #[serde(flatten)] inner: Box<Identify> },
            2 => Resume {
                session: Snowflake,
            },
            3 => SetPresence { #[serde(flatten)] inner: Box<SetPresence> }
        }

        #[cfg(test)]
        mod tests {
            use std::mem::size_of;

            use super::*;

            #[test]
            fn test_client_msg_size() {
                assert_eq!(16, size_of::<Message>());
            }
        }
    }
}
