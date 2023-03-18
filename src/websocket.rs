use std::sync::Arc;

use chatgpt::prelude::{ChatGPT, Conversation};
use eyre::Context;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite;
use tracing::Instrument;
use twitch_api::{
    eventsub::{
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event, NotificationMetadata, Payload, channel::{ChannelFollowV2, ChannelFollowV2Payload}, Message,
    },
    types::{self},
    HelixClient,
};
use twitch_oauth2::UserToken;

pub struct WebsocketClient {
    pub session_id: Option<String>,
    pub token: Arc<RwLock<UserToken>>,
    pub client: HelixClient<'static, reqwest::Client>,
    pub user_id: types::UserId,
    pub connect_url: url::Url,
    pub chat_gpt: ChatGPT,
}

impl WebsocketClient {
    pub async fn connect(
        &self,
    ) -> Result<
            tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        eyre::Error,
    > {
        tracing::info!("connecting to twitch");
        let config = tungstenite::protocol::WebSocketConfig {
            max_send_queue: None,
            max_message_size: Some(64 << 20), // 64 MiB
            max_frame_size: Some(16 << 20),   // 16 MiB
            accept_unmasked_frames: false,
        };
        let (socket, _) =
            tokio_tungstenite::connect_async_with_config(&self.connect_url, Some(config))
                .await
                .context("Can't connect")?;

        Ok(socket)
    }

    #[tracing::instrument(name = "subscriber", skip_all, fields())]
    pub async fn run(mut self, _opts: &crate::Opts) -> Result<(), eyre::Error> {
        let mut s = self
            .connect()
            .await
            .context("when establishing connection")?;
        loop {
            tokio::select!(
            Some(msg) = futures::StreamExt::next(&mut s) => {
                let span = tracing::info_span!("message received", raw_message = ?msg);
                let msg = match msg {
                    Err(tungstenite::Error::Protocol(
                        tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                    )) => {
                        tracing::warn!(
                            "connection was sent an unexpected frame or was reset, reestablishing it"
                        );
                        s = self
                            .connect().instrument(span)
                            .await
                            .context("when reestablishing connection")?;
                        continue
                    }
                    _ => msg.context("when getting message")?,
                };
                self.process_message(msg).instrument(span).await?
            })
        }
    }

    pub async fn process_message(&mut self, msg: tungstenite::Message) -> Result<(), eyre::Report> {
        match msg {
            tungstenite::Message::Text(s) => {
                tracing::info!("{s}");
                match Event::parse_websocket(&s)? {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.process_welcome_message(session).await?;
                        Ok(())
                    }
                    EventsubWebsocketData::Notification {
                        metadata: metadata,
                        payload: event,
                    } => {
                        self.process_notification(event, metadata).await?;
                        Ok(())
                    },
                    EventsubWebsocketData::Revocation {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    _ => Ok(()),
                }
            }
            tungstenite::Message::Close(_) => todo!(),
            _ => Ok(()),
        }
    }

    async fn process_notification(&self, data: Event, metadata: NotificationMetadata<'_>) -> Result<(), eyre::Report> {
        match data {
            Event::ChannelFollowV2(Payload{
                message:
                    Message::Notification(ChannelFollowV2Payload {
                        user_name,
                        ..
                    }),
                ..
            }) => {
                self.process_new_follow(user_name.to_string()).await?;
                Ok(())
            },
            _ => Ok(()),
        }
    }

    async fn process_new_follow(&self, payload: String) -> Result<(), eyre::Report> {
        let mut conversation: Conversation = self.chat_gpt.new_conversation_directed(
            "You are NullGPT, when answering any questions, you always answer with a short epic story involving the Rust programming language and null."
        );
    
        // Sending messages to the conversation
        let response = conversation
            .send_message(format!("tell me a short story about my new subscriber {}?", payload))
            .await?;
    
        println!("Response: {}", response.message().content);
        Ok(())
    }

    pub async fn process_welcome_message(
        &mut self,
        data: SessionData<'_>,
    ) -> Result<(), eyre::Report> {
        self.session_id = Some(data.id.to_string());
        if let Some(url) = data.reconnect_url {
            self.connect_url = url.parse()?;
        }
        let req = twitch_api::helix::eventsub::CreateEventSubSubscriptionRequest::new();
        let body = twitch_api::helix::eventsub::CreateEventSubSubscriptionBody::new(
            twitch_api::eventsub::channel::ChannelFollowV2::new(self.user_id.clone(),self.user_id.clone()),
            twitch_api::eventsub::Transport::websocket(data.id.clone()),
        );
        
        self.client
            .req_post(req, body, &*self.token.read().await)
            .await?;
        tracing::info!("listening to ban and unbans");
        Ok(())
    }
}
