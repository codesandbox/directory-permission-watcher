use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{thread, time};

use crate::common_path::common_path_all;

mod common_path;
mod permissions;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            match tx.send(res).await {
                Ok(()) => {}
                Err(err) => println!("watch error: {:?}", err),
            };
        })
    })?;

    Ok((watcher, rx))
}

async fn async_watch(path: PathBuf) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    thread::sleep(time::Duration::from_millis(500));

    permissions::check_permission_recursive(path.clone());

    let common_path: Arc<Mutex<Option<PathBuf>>> = Default::default();
    let common_path_ref = common_path.clone();
    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_millis(5000));

        let mut paths_mutex = common_path_ref.lock().unwrap();
        let taken_path = paths_mutex.take();
        if let Some(path) = taken_path {
            permissions::check_permission_recursive(path);
        }
    });

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if event.kind.is_create() || event.kind.is_modify() || event.kind.is_other() {
                    let mut paths: Vec<PathBuf> = event.clone().paths.clone();
                    let mut common_path_mutex = common_path.lock().unwrap();
                    if let Some(prev_common_path) = common_path_mutex.take() {
                        paths.push(prev_common_path);
                    }
                    let new_common_path = common_path_all(paths);
                    *common_path_mutex = new_common_path;
                }
            }
            Err(err) => println!("watch error: {:?}", err),
        }
    }

    Ok(())
}

fn start_watcher(path: PathBuf) {
    println!("watching: {:?}", path.clone());

    futures::executor::block_on(async {
        if let Err(e) = async_watch(path.clone()).await {
            println!("error: {:?}", e)
        }
    });

    // Restart watcher every time it fails with a certain delay
    thread::sleep(time::Duration::from_secs(60));
    start_watcher(path.clone());
}

fn main() {
    let path_input = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    let path = Path::new(path_input).to_path_buf();

    start_watcher(path);
}
