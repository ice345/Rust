// TODO: fix the `assert_eq` at the end of the tests.
//  Do you understand why that's the resulting output?
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

// 服务器超时触发 → 强制取消 read_to_end → 丢弃 stream → 连接关闭。

/**
客户端行为              服务器行为
│                         │
├─ 新建连接 ──────────────→ 接受连接
│                         │
├─ 发送 "he" ────────────→ 读取 "he"
│                         │
├─ sleep 40ms             │ 等待后续数据...
│                         │ (20ms 后超时触发)
│                         │ 关闭连接
│                         │
├─ sleep 结束             │ 
│                         │
├─ 尝试发送 "llo" ────────→ 连接已关闭，写入失败
│                         │
└─ 关闭写入端             │ 
*/

// 处理网络连接
pub async fn run(listener: TcpListener, n_messages: usize, timeout: Duration) -> Vec<u8> {
    let mut buffer = Vec::new();
    for _ in 0..n_messages {
        let (mut stream, _) = listener.accept().await.unwrap();
        let _ = tokio::time::timeout(timeout, async {
            stream.read_to_end(&mut buffer).await.unwrap();
        })
        .await;  // 超时机制
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn ping() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let messages = vec!["hello", "from", "this", "task"];
        let timeout = Duration::from_millis(20);
        let handle = tokio::spawn(run(listener, messages.len(), timeout.clone())); //启动run函数作为一个新的异步任务

        for message in messages {
            let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
            let (_, mut writer) = socket.split();  // 将连接分为读取和写入

            let (beginning, end) = message.split_at(message.len() / 2);  //将消息一分为二

            // Send first half
            writer.write_all(beginning.as_bytes()).await.unwrap();
            tokio::time::sleep(timeout * 2).await;  // 等待timeout的两倍,所以就会超时
            writer.write_all(end.as_bytes()).await.unwrap();

            // Close the write side of the socket
            let _ = writer.shutdown().await;
        }

        let buffered = handle.await.unwrap();
        let buffered = std::str::from_utf8(&buffered).unwrap();
        assert_eq!(buffered, "hefrthta");
    }
}
