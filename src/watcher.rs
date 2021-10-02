use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
};
use notify::{
    Error as NotifyError, Event, FsEventWatcher, RecommendedWatcher, RecursiveMode,
    Watcher as NotifyWatcher,
};
use std::{convert::TryInto, path::PathBuf};

pub struct Watcher {
    notify_instance: FsEventWatcher,
    pub notify_rx: Receiver<String>,
}

impl Watcher {
    pub fn new() -> Self {
        let (tx, rx) = channel::<String>(12);

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let notify_instance = RecommendedWatcher::new(move |res: Result<Event, NotifyError>| {
            futures::executor::block_on(async {
                let s: String = res.unwrap().paths[0]
                    .as_path()
                    .to_str()
                    .unwrap()
                    .to_string();
                match tx.send(s).await {
                    Ok(()) => {}
                    Err(err) => println!("watch error: {:?}", err),
                };
            })
        })
        .unwrap();

        Self {
            notify_instance,
            notify_rx: rx,
        }
    }

    // TODO: Figure out callbacks...
    pub fn watch(&mut self, path_str: &str) {
        let path: PathBuf = path_str.try_into().unwrap();
        self.notify_instance
            .watch(&path.as_path(), RecursiveMode::Recursive)
            .unwrap();
    }
}
