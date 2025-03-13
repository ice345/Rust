// TODO: Replace `Mutex` with `RwLock` in the `TicketStore` struct and
//  all other relevant places to allow multiple readers to access the ticket store concurrently.

/**

### **拆解代码逻辑**
1. **创建消息通道**  
    ```rust
    let (response_sender, response_receiver) = sync_channel(1);
    ```
这里 `sync_channel(1)` 创建了一个有**容量为1**的同步通道，`response_sender` 用于发送数据，`response_receiver` 用于接收数据。

2. **发送消息**  
    ```rust
    self.sender.try_send(Command::Insert {
        draft,
        response_channel: response_sender,
    })
    ```
    `self.sender` 是一个 `mpsc::Sender<Command>`，它向内部的**任务处理进程**（服务器端）发送 `Command::Insert` 命令，  
    - `draft` 是插入的数据  
    - `response_sender` 发送端被一同传递，使得**服务器端处理完后可以用它回传结果**  
    - `try_send` 失败时返回 `OverloadedError`，表示消息队列已满

3. **等待并获取服务器端的响应**  
    ```rust
    Ok(response_receiver.recv().unwrap())
    ```
    `recv()` 让调用者**阻塞等待**，直到 `response_sender` 发送回 `TicketId`。  
    这意味着：
    - `self.sender` 关联的服务器（后台线程）接收到 `Command::Insert` 后，**会进行插入操作**  
    - 服务器端处理完后，通过 `response_sender.send(ticket_id)` 发送结果  
    - `response_receiver.recv()` 从通道接收数据并返回

### **类比服务器进程**
这里 `self.sender` 指向的是某个**后台线程（或 actor 进程）**，它：
- 维护着一个 `Receiver<Command>` 的消息队列  
- 以 `loop` 监听消息，接收到 `Command::Insert` 后执行插入逻辑  
- 通过 `response_sender.send(result)` 发送处理结果

**可以类比为：**
- **客户端**：`insert()` 方法，相当于请求 API  
- **服务器**：后台线程，监听 `self.sender` 发送的命令并执行逻辑  
- **通信方式**：Rust 的 `mpsc`（多生产者-单消费者）或 `sync_channel` 实现的**消息传递**

*/


use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, RwLock};

use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>, //服务器端的任务处理进程
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, OverloadedError> {
        let (response_sender, response_receiver) = sync_channel(1);
        self.sender
            .try_send(Command::Insert {
                draft,
                response_channel: response_sender,
            })
            .map_err(|_| OverloadedError)?;
        Ok(response_receiver.recv().unwrap())  //等待并获取服务器端的响应
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Arc<RwLock<Ticket>>>, OverloadedError> {
        let (response_sender, response_receiver) = sync_channel(1);
        self.sender
            .try_send(Command::Get {
                id,
                response_channel: response_sender,
            })
            .map_err(|_| OverloadedError)?;
        Ok(response_receiver.recv().unwrap())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The store is overloaded")]
pub struct OverloadedError;

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Arc<RwLock<Ticket>>>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {  //loop监听信息
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                let _ = response_channel.send(id);
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                let _ = response_channel.send(ticket);
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
