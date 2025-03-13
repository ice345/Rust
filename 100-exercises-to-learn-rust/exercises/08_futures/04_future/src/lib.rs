//! TODO: get the code to compile by **re-ordering** the statements
//!  in the `example` function. You're not allowed to change the
//!  `spawner` function nor what each line does in `example`.
//!   You can wrap existing statements in blocks `{}` if needed.
use std::sync::Arc;
use std::rc::Rc;
use tokio::task::yield_now;

fn spawner() {
    tokio::spawn(example());
}

async fn example() {
    // let non_send = Arc::new(1);
    // yield_now().await;
    // println!("{}", non_send);

    // other solution
    {
        let non_send = Rc::new(1);
        println!("{}", non_send);
    } // 提前释放
    yield_now().await; //因为这个会将Future转移到其他线程继续执行,所以会Send,但如果结构中一部分没有实现Send就会报错
}
