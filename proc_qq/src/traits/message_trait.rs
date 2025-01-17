use async_trait::async_trait;
use rq_engine::msg::elem::{FlashImage, FriendImage, GroupImage, Text};
use rq_engine::msg::MessageChain;
use rq_engine::pb::msg::elem::Elem;
use rq_engine::structs::{FriendMessage, GroupMessage, MessageReceipt, TempMessage};
use rq_engine::{RQError, RQResult};
use rs_qq::client::event::{FriendMessageEvent, GroupMessageEvent, TempMessageEvent};
use rs_qq::structs::Group;
use std::sync::Arc;
use std::time::Duration;

use crate::{ClientTrait, MessageEvent};

pub enum MessageTarget {
    // Group(group_code,uin)
    Group(i64, i64),
    // Private(uin)
    Private(i64),
    // Temp(group_code,uin)
    Temp(Option<i64>, i64),
}

pub enum UploadImage {
    FriendImage(FriendImage),
    GroupImage(GroupImage),
}

impl Into<Vec<Elem>> for UploadImage {
    fn into(self) -> Vec<Elem> {
        match self {
            UploadImage::FriendImage(i) => i.into(),
            UploadImage::GroupImage(i) => i.into(),
        }
    }
}

pub trait MessageTargetTrait: Send + Sync {
    fn target(&self) -> MessageTarget;
}

pub trait MessageContentTrait: Send + Sync {
    fn message_content(&self) -> String;
}

#[async_trait]
pub trait MessageSendToSourceTrait: Send + Sync + ClientTrait {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt>;

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage>;

    async fn send_audio_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
        codec: u32,
        audio_duration: Duration,
    ) -> RQResult<MessageReceipt>;
}

pub trait TextEleParseTrait {
    fn parse_text(self) -> Text;
}

pub trait MessageChainParseTrait {
    fn parse_message_chain(self) -> MessageChain;
}

impl MessageContentTrait for MessageChain {
    fn message_content(&self) -> String {
        self.to_string()
    }
}

impl MessageTargetTrait for GroupMessage {
    fn target(&self) -> MessageTarget {
        MessageTarget::Group(self.group_code, self.from_uin)
    }
}

impl MessageContentTrait for GroupMessage {
    fn message_content(&self) -> String {
        self.elements.message_content()
    }
}

impl MessageTargetTrait for GroupMessageEvent {
    fn target(&self) -> MessageTarget {
        self.message.target()
    }
}

impl MessageContentTrait for GroupMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl ClientTrait for GroupMessageEvent {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(source, message).await
    }

    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        self.client.must_find_group(group_code, auto_reload).await
    }

    async fn bot_uin(&self) -> i64 {
        self.client.bot_uin().await
    }
}

#[async_trait]
impl MessageSendToSourceTrait for GroupMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(self, message).await
    }

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        Ok(UploadImage::GroupImage(
            self.client
                .upload_group_image(self.message.group_code, data.into())
                .await?,
        ))
    }

    async fn send_audio_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
        codec: u32,
        _audio_duration: Duration,
    ) -> RQResult<MessageReceipt> {
        let group_audio = self
            .client
            .upload_group_audio(self.message.group_code, data.into(), codec)
            .await?;
        self.client
            .send_group_audio(self.message.group_code, group_audio)
            .await
    }
}

impl MessageTargetTrait for FriendMessage {
    fn target(&self) -> MessageTarget {
        MessageTarget::Private(self.from_uin)
    }
}

impl MessageContentTrait for FriendMessage {
    fn message_content(&self) -> String {
        self.elements.to_string()
    }
}

impl MessageTargetTrait for FriendMessageEvent {
    fn target(&self) -> MessageTarget {
        self.message.target()
    }
}

impl MessageContentTrait for FriendMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl ClientTrait for FriendMessageEvent {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(source, message).await
    }

    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        self.client.must_find_group(group_code, auto_reload).await
    }

    async fn bot_uin(&self) -> i64 {
        self.client.bot_uin().await
    }
}

#[async_trait]
impl MessageSendToSourceTrait for FriendMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(self, message).await
    }

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        Ok(UploadImage::FriendImage(
            self.client
                .upload_friend_image(self.message.from_uin, data.into())
                .await?,
        ))
    }

    async fn send_audio_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
        _codec: u32,
        audio_duration: Duration,
    ) -> RQResult<MessageReceipt> {
        let friend_audio = self
            .client
            .upload_friend_audio(self.message.from_uin, data.into(), audio_duration)
            .await?;
        self.client
            .send_friend_audio(self.message.from_uin, friend_audio)
            .await
    }
}

impl MessageTargetTrait for TempMessage {
    fn target(&self) -> MessageTarget {
        MessageTarget::Temp(self.group_code, self.from_uin)
    }
}

impl MessageContentTrait for TempMessage {
    fn message_content(&self) -> String {
        self.elements.to_string()
    }
}

impl MessageTargetTrait for TempMessageEvent {
    fn target(&self) -> MessageTarget {
        self.message.target()
    }
}

impl MessageContentTrait for TempMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl ClientTrait for TempMessageEvent {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(source, message).await
    }

    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        self.client.must_find_group(group_code, auto_reload).await
    }

    async fn bot_uin(&self) -> i64 {
        self.client.bot_uin().await
    }
}

#[async_trait]
impl MessageSendToSourceTrait for TempMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_target(self, message).await
    }

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        _: S,
    ) -> RQResult<UploadImage> {
        RQResult::Err(RQError::Other(
            "tmp message not supported upload image".to_owned(),
        ))
    }

    async fn send_audio_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        _data: S,
        _codec: u32,
        _audio_duration: Duration,
    ) -> RQResult<MessageReceipt> {
        RQResult::Err(RQError::Other(
            "tmp message not supported upload audio".to_owned(),
        ))
    }
}

impl MessageTargetTrait for MessageEvent {
    fn target(&self) -> MessageTarget {
        match self {
            MessageEvent::GroupMessage(event) => event.target(),
            MessageEvent::FriendMessage(event) => event.target(),
            MessageEvent::TempMessage(event) => event.target(),
        }
    }
}

impl MessageContentTrait for MessageEvent {
    fn message_content(&self) -> String {
        match self {
            MessageEvent::GroupMessage(event) => event.message_content(),
            MessageEvent::FriendMessage(event) => event.message_content(),
            MessageEvent::TempMessage(event) => event.message_content(),
        }
    }
}

#[async_trait]
impl ClientTrait for MessageEvent {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client().send_message_to_target(source, message).await
    }

    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        self.client().must_find_group(group_code, auto_reload).await
    }

    async fn bot_uin(&self) -> i64 {
        self.client().bot_uin().await
    }
}

#[async_trait]
impl MessageSendToSourceTrait for MessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        match self {
            MessageEvent::GroupMessage(event) => event.send_message_to_source(message),
            MessageEvent::FriendMessage(event) => event.send_message_to_source(message),
            MessageEvent::TempMessage(event) => event.send_message_to_source(message),
        }
        .await
    }

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        match self {
            MessageEvent::GroupMessage(event) => event.upload_image_to_source(data),
            MessageEvent::FriendMessage(event) => event.upload_image_to_source(data),
            MessageEvent::TempMessage(event) => event.upload_image_to_source(data),
        }
        .await
    }

    async fn send_audio_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
        codec: u32,
        audio_duration: Duration,
    ) -> RQResult<MessageReceipt> {
        match self {
            MessageEvent::GroupMessage(event) => {
                event.send_audio_to_source(data, codec, audio_duration)
            }
            MessageEvent::FriendMessage(event) => {
                event.send_audio_to_source(data, codec, audio_duration)
            }
            MessageEvent::TempMessage(event) => {
                event.send_audio_to_source(data, codec, audio_duration)
            }
        }
        .await
    }
}

impl TextEleParseTrait for String {
    fn parse_text(self) -> Text {
        Text::new(self)
    }
}

impl TextEleParseTrait for &str {
    fn parse_text(self) -> Text {
        Text::new(self.to_owned())
    }
}

impl MessageChainParseTrait for String {
    fn parse_message_chain(self) -> MessageChain {
        MessageChain::new(self.parse_text())
    }
}

impl MessageChainParseTrait for &str {
    fn parse_message_chain(self) -> MessageChain {
        MessageChain::new(self.parse_text())
    }
}

impl MessageChainParseTrait for FriendImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for GroupImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for FlashImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for UploadImage {
    fn parse_message_chain(self) -> MessageChain {
        match self {
            UploadImage::FriendImage(i) => i.parse_message_chain(),
            UploadImage::GroupImage(i) => i.parse_message_chain(),
        }
    }
}
