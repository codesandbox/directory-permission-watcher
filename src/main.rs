use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time;
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

async fn async_watch(path: &Path) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    tokio::time::sleep(time::Duration::from_millis(500)).await;
    chmod::update_permission_recursive(path);

    // Spawn the watcher in a new thread so it can't block the main thread.
    let thread = tokio::spawn(async move {
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
    });

    if let Err(err) = thread.await {
        println!("watch thread join err: {:?}", err);
    };

    Ok(())
}

async fn start_watcher(path: &Path) {
    println!("Starting watcher in: {:?}", path);

    if let Err(e) = async_watch(path).await {
        println!("Watcher crashed: {:?}", e)
    }
}

#[tokio::main]
async fn main() {
    let path_input = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    let path = Path::new(path_input).to_path_buf();

    loop {
        start_watcher(&path).await;
        // Restart watcher every time it fails
        tokio::time::sleep(time::Duration::from_secs(30)).await;
    }
}
