RUST_PROC_QQ
============

- 一个开箱即用, 使用简单的, Rust语言的QQ机器人框架. (基于[RS-QQ](https://github.com/lz1998/rs-qq))
- 参考了Spring(java)和Rocket(rust)的思想, 如果您使用此类框架将很好上手

## 框架目的

- 简单化 : 让程序员写更少的代码
    - 自动管理客户端生命周期以及TCP重连
    - 封装登录流程, 自动获取ticket, 验证滑动条
- 模块化 : 让调理更清晰
    - 模块化, 实现插件之间的分离, 更好的启用禁用

## 如何使用 / demo

### 引用

Cargo.toml

```toml
proc_qq = { git = "https://github.com/niuhuan/rust_proc_qq.git", branch = "master" }
```

### 声明一个模块

hello_module.rs

```rust
use proc_qq::re_export::rs_qq::client::event::GroupMessageEvent;
use proc_qq::re_export::rs_qq::msg::elem::Text;
use proc_qq::re_export::rs_qq::msg::MessageChain;
use proc_qq::{event, module, ClientTrait, MessageEvent, MessageTrait, Module};

/// 监听群消息
/// 使用event宏进行声明监听消息
/// 参数为rs-qq支持的任何一个类型的消息事件, 必须是引用.
/// 返回值为 anyhow::Result<bool>, Ok(true)为拦截事件, 不再向下一个监听器传递
#[event]
async fn print(event: &MessageEvent) -> anyhow::Result<bool> {
    if content.eq("你好") {
        event
            .client()
            .send_message_to_source(event, MessageChain::new(Text::new("世界".to_owned())))
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        event
            .client()
            .send_message_to_source(event, MessageChain::new(Text::new("NB".to_owned())))
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[event]
async fn group_hello(_: &GroupMessageEvent) -> anyhow::Result<bool> {
    Ok(false)
}

/// 返回一个模块 (向过程宏改进中)
pub(crate) fn module() -> Module {
    // id, name, [plugins ...]
    module!("hello", "你好", print, group_hello)
}
```

### 启动

main.rs

```rust
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::Authentication::{QRCode, UinPassword};
use proc_qq::ClientBuilder;

mod hello_module;

/// 启动并使用为二维码登录
#[tokio::test]
async fn test_qr_login() {
    // 初始化日志打印
    init_tracing_subscriber();
    // 使用builder创建
    ClientBuilder::new()
        .priority_session("session.token")      // 默认使用session.token登录
        // .device(JsonFile("device.json")) // 设备默认值 
        .authentication(QRCode)                 // 若不成功则使用二维码登录
        .build(vec![hello_module::module()])    // 您可以注册多个模块
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
}

fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("rs_qq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                .with_target("proc_qq_examples", Level::DEBUG),
        )
        .init();
}

```

### 效果

![demo](images/demo_01.jpg)

## 功能

### 支持的事件

```rust
use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, PrivateMessageEvent, TempMessageEvent,
};
use proc_qq::{MessageEvent, };
```

同时支持多种消息的事件封装中...

### 拓展

#### 直接获取消息的正文内容

```rust
use prco_qq::MessageTrait;

let private_message_event: PrivateMessageEvent = _;

private_message_event.message_content();
```

#### 直接回复消息到消息源

```rust
use prco_qq::MessageTrait;
use prco_qq::ClientTrait;

let group_message_event: & GroupMessageEvent = _;
let client: Arc<Client> = _;

client.send_message_to_source(group_message_event, my_message);
```

## 其他

[使用此框架的模版](proc_qq_template)

[例子](proc_qq_examples)
