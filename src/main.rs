use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    panic,
    path::{Path, PathBuf},
};
use std::{thread, time};
use tokio::sync::mpsc::{channel, Receiver};

mod chmod;
mod permissions;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(512);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        match tx.blocking_send(res) {
            Ok(()) => {}
            Err(err) => println!("watch error: {:?}", err),
        };
    })?;

    Ok((watcher, rx))
}

async fn async_watch(path: PathBuf) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    thread::sleep(time::Duration::from_millis(500));

    chmod::update_permission_recursive(path.clone());

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if event.kind.is_create() || event.kind.is_modify() || event.kind.is_other() {
                    if cfg!(debug_assertions) {
                        println!("watch event: {:?}", event.kind);
                    }

                    permissions::check_permissions(event.paths);
                }
            }
            Err(err) => println!("watch error: {:?}", err),
        }
    }

    Ok(())
}

fn start_watcher(path: PathBuf) {
    println!("Starting watcher in: {:?}", path);

    let res = panic::catch_unwind(|| {
        futures::executor::block_on(async {
            if let Err(e) = async_watch(path.clone()).await {
                println!("error: {:?}", e)
            }
        });
    });

    if let Err(err) = res {
        println!("Watcher crashed {:?}", err);
    }

    // Restart watcher every time it fails
    thread::sleep(time::Duration::from_secs(30));
    start_watcher(path.clone());
}

fn main() {
    let path_input = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    let path = Path::new(path_input).to_path_buf();

    start_watcher(path);
}
