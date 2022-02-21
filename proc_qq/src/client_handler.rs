use std::sync::Arc;

use async_trait::async_trait;
use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, PrivateMessageEvent, TempMessageEvent,
};
use rs_qq::handler::{Handler, QEvent};

pub struct ClientHandler {
    pub(crate) modules: Vec<Module>,
}

impl ClientHandler {}

enum MapResult<'a> {
    None,
    Process(&'a str, &'a str),
    Exception(&'a str, &'a str),
}

macro_rules! map_handlers {
    ($self:expr $(,$event:expr, $process:path)* $(,)?) => {{
        let mut result = MapResult::None;
        for m in &$self.modules {
            for h in &m.handles {
                match &h.process {
                    $(
                    $process(e) => match e.handle(&$event).await {
                        Ok(b) => {
                            if b {
                                result = MapResult::Process(&m.id, &h.name);
                            }
                        }
                        Err(err) => {
                            tracing::error!(target = "proc_qq", " 出现错误 : {:?}", err);
                            result = MapResult::Exception(&m.id, &h.name);
                        }
                    },
                    )*
                    _ => (),
                }
                if let MapResult::None = result {
                } else {
                    break;
                }
            }
            if let MapResult::None = result {
            } else {
                break;
            }
        }
        result
    }};
}

#[async_trait]
impl Handler for ClientHandler {
    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::GroupMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(GROUP={}, UIN={}) MESSAGE : {}",
                    event.message.group_code,
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::GroupMessage(&event);
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::PrivateMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(UIN={}) MESSAGE : {}",
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::PrivateMessage(&event);
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::PrivateMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::TempMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(UIN={}) MESSAGE : {}",
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::TempMessage(&event);
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::TempMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::GroupRequest(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "REQUEST (GROUP={}, UIN={}): {}",
                    event.request.group_code,
                    event.request.req_uin,
                    event.request.message,
                );
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupRequest);
            }
            QEvent::FriendRequest(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "REQUEST (UIN={}): {}",
                    event.request.req_uin,
                    event.request.message
                );
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendRequest);
            }
            QEvent::NewFriend(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::NewFriend);
            }
            QEvent::FriendPoke(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendPoke);
            }
            QEvent::DeleteFriend(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::DeleteFriend);
            }
            QEvent::GroupMute(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupMute);
            }
            QEvent::GroupLeave(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupLeave);
            }
            QEvent::GroupNameUpdate(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupNameUpdate);
            }
            QEvent::GroupMessageRecall(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupMessageRecall);
            }
            QEvent::FriendMessageRecall(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendMessageRecall);
            }
            _ => tracing::debug!(target = "proc_qq", "{:?}", e),
        }
    }
}

pub struct Module {
    pub id: String,
    pub name: String,
    pub handles: Vec<ModuleEventHandler>,
}

#[macro_export]
macro_rules! module {
    ($id:expr,$name:expr $(, $x:tt)* $(,)?) => (
        ::proc_qq::Module {
            id: $id.to_owned(),
            name: $name.to_owned(),
            handles: vec![$($x {}.into(),)*],
        }
    );
}

pub struct ModuleEventHandler {
    pub name: String,
    pub process: ModuleEventProcess,
}

pub enum ModuleEventProcess {
    GroupMessage(Box<dyn GroupMessageEventProcess>),
    PrivateMessage(Box<dyn PrivateMessageEventProcess>),
    TempMessage(Box<dyn TempMessageEventProcess>),
    GroupRequest(Box<dyn GroupRequestEventProcess>),
    FriendRequest(Box<dyn FriendRequestEventProcess>),

    NewFriend(Box<dyn NewFriendEventProcess>),
    FriendPoke(Box<dyn FriendPokeEventProcess>),
    DeleteFriend(Box<dyn DeleteFriendEventProcess>),

    GroupMute(Box<dyn GroupMuteEventProcess>),
    GroupLeave(Box<dyn GroupLeaveEventProcess>),
    GroupNameUpdate(Box<dyn GroupNameUpdateEventProcess>),

    GroupMessageRecall(Box<dyn GroupMessageRecallEventProcess>),
    FriendMessageRecall(Box<dyn FriendMessageRecallEventProcess>),

    Message(Box<dyn MessageEventProcess>),
}

macro_rules! process_trait {
    ($name:ident, $event:path) => {
        #[async_trait]
        pub trait $name: Sync + Send {
            async fn handle(&self, event: &$event) -> anyhow::Result<bool>;
        }
    };
}

process_trait!(GroupMessageEventProcess, GroupMessageEvent);
process_trait!(PrivateMessageEventProcess, PrivateMessageEvent);
process_trait!(TempMessageEventProcess, TempMessageEvent);

process_trait!(GroupRequestEventProcess, GroupRequestEvent);
process_trait!(FriendRequestEventProcess, FriendRequestEvent);

process_trait!(NewFriendEventProcess, NewFriendEvent);
process_trait!(FriendPokeEventProcess, FriendPokeEvent);
process_trait!(DeleteFriendEventProcess, DeleteFriendEvent);

process_trait!(GroupMuteEventProcess, GroupMuteEvent);
process_trait!(GroupLeaveEventProcess, GroupLeaveEvent);
process_trait!(GroupNameUpdateEventProcess, GroupNameUpdateEvent);

process_trait!(GroupMessageRecallEventProcess, GroupMessageRecallEvent);
process_trait!(FriendMessageRecallEventProcess, FriendMessageRecallEvent);

pub enum MessageEvent<'a> {
    GroupMessage(&'a GroupMessageEvent),
    PrivateMessage(&'a PrivateMessageEvent),
    TempMessage(&'a TempMessageEvent),
}

impl MessageEvent<'_> {
    pub fn client(&self) -> Arc<rs_qq::Client> {
        match self {
            MessageEvent::GroupMessage(e) => e.client.clone(),
            MessageEvent::PrivateMessage(e) => e.client.clone(),
            MessageEvent::TempMessage(e) => e.client.clone(),
        }
    }
}

process_trait!(MessageEventProcess, MessageEvent);
