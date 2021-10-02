use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use notify::{
    Error as NotifyError, Event, FsEventWatcher, RecommendedWatcher, RecursiveMode,
    Watcher as NotifyWatcher,
};
use std::path::PathBuf;

use crate::error::NodeWatcherError;

pub struct Watcher {
    notify_instance: FsEventWatcher,
}

impl Watcher {
    pub fn new(callback: ThreadsafeFunction<String>) -> Result<Self, NodeWatcherError> {
        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let notify_instance = RecommendedWatcher::new(move |res: Result<Event, NotifyError>| {
            futures::executor::block_on(async {
                callback.call(
                    Watcher::wrap_napi_process_watcher_event(res),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            })
        })?;

        Ok(Self { notify_instance })
    }

    fn wrap_napi_process_watcher_event(
        res: Result<Event, NotifyError>,
    ) -> Result<String, napi::Error> {
        Ok(Watcher::process_watcher_event(res)?)
    }

    // TODO: Create some event struct that can be converted to a js object using serde...
    fn process_watcher_event(res: Result<Event, NotifyError>) -> Result<String, NodeWatcherError> {
        let event = res?;
        let path = event.paths[0].as_path();
        if let Some(path_str) = path.to_str() {
            Ok(path_str.to_string())
        } else {
            Ok("Some error".to_string())
        }
    }

    pub fn watch(&mut self, path_str: &str) -> Result<(), NodeWatcherError> {
        let path: PathBuf = PathBuf::from(path_str);
        self.notify_instance
            .watch(&path.as_path(), RecursiveMode::Recursive)?;
        Ok(())
    }
}
