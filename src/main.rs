use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::{
    collections::HashSet,
    iter::FromIterator,
    sync::{Arc, Mutex},
};
use std::{thread, time};

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

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    let paths: Arc<Mutex<Option<HashSet<PathBuf>>>> = Default::default();
    let paths_ref = paths.clone();
    thread::spawn(move || loop {
        let delay_duration = time::Duration::from_secs(5);
        thread::sleep(delay_duration);

        let mut paths_mutex = paths_ref.lock().unwrap();
        let taken_paths = paths_mutex.take();
        if let Some(paths) = taken_paths {
            if paths.len() > 0 {
                let paths_vec = Vec::from_iter(paths);
                permissions::check_permissions(paths_vec);
            }
        }
    });

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if !event.kind.is_remove() && !event.kind.is_access() && !event.kind.is_other() {
                    if cfg!(debug_assertions) {
                        println!("watch event: {:?}", event.kind);
                    }

                    let mut paths_mutex = paths.lock().unwrap();
                    let mut taken_paths = paths_mutex.take();
                    if taken_paths == None {
                        taken_paths = Some(Default::default());
                    }

                    if let Some(mut paths) = taken_paths.clone() {
                        for path in event.clone().paths {
                            paths.insert(path.to_path_buf());
                        }

                        *paths_mutex = Some(paths);
                    } else {
                        *paths_mutex = taken_paths;
                    }

                    println!("{:?}", paths_mutex);
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
    let restart_delay = time::Duration::from_secs(60);
    thread::sleep(restart_delay);
    start_watcher(path.clone());
}

fn main() {
    let path_input = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    let path = Path::new(path_input).to_path_buf();

    permissions::check_permission_recursive(path.clone());
    start_watcher(path.clone());
}
