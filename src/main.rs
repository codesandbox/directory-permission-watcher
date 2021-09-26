use file_mode::{ModePath, ProtectionBit, User};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

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

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if !event.kind.is_remove() {
                    if cfg!(debug_assertions) {
                        println!("watch event: {:?}", event.kind);
                    }
                    check_permissions(event.clone().paths);
                }
            }
            Err(err) => println!("watch error: {:?}", err),
        }
    }

    Ok(())
}

fn check_permissions(paths: Vec<PathBuf>) {
    for path in paths.clone().iter() {
        let p = path.as_path();

        if cfg!(debug_assertions) {
            println!("Validating file permissions of {:?}", p);
        }

        match p.mode() {
            Ok(mut mode) => {
                if let Some(file_type) = mode.file_type() {
                    if file_type.is_directory() || file_type.is_regular_file() {
                        let mut should_update_path = false;

                        let mut owner_protection = mode.user_protection(User::Owner);
                        let mut group_protection = mode.user_protection(User::Group);
                        let mut other_protection = mode.user_protection(User::Other);

                        // Owner permissions
                        if !owner_protection.is_read_set() {
                            should_update_path = true;
                            owner_protection.set(ProtectionBit::Read)
                        }
                        if !owner_protection.is_write_set() {
                            should_update_path = true;
                            owner_protection.set(ProtectionBit::Write)
                        }

                        // Group permissions
                        if !group_protection.is_read_set() {
                            should_update_path = true;
                            group_protection.set(ProtectionBit::Read)
                        }
                        if !group_protection.is_write_set() {
                            should_update_path = true;
                            group_protection.set(ProtectionBit::Write)
                        }

                        // Other permissions
                        if !other_protection.is_read_set() {
                            should_update_path = true;
                            other_protection.set(ProtectionBit::Read)
                        }

                        // Directory needs execute permissions as well
                        if file_type.is_directory() {
                            if !owner_protection.is_execute_set() {
                                should_update_path = true;
                                owner_protection.set(ProtectionBit::Execute)
                            }
                            if !group_protection.is_execute_set() {
                                should_update_path = true;
                                group_protection.set(ProtectionBit::Execute)
                            }
                            if !other_protection.is_execute_set() {
                                should_update_path = true;
                                other_protection.set(ProtectionBit::Execute)
                            }
                        }

                        if should_update_path {
                            mode.set_protection(User::Owner, &owner_protection);
                            mode.set_protection(User::Group, &group_protection);
                            mode.set_protection(User::Other, &other_protection);

                            match p.set_mode(mode) {
                                Ok(_) => println!("Updated file permissions of {:?}", path),
                                Err(err) => println!("Could not update file permissions {:?}", err),
                            }
                        }
                    }
                }
            }
            Err(err) => println!("Could not load permissions of {:?}, {:?}", p, err),
        }
    }
}

fn start_watcher(path: &str) {
    println!("watching: {}", path);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(path).await {
            println!("error: {:?}", e)
        }
    });
}

fn main() {
    let path = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    start_watcher(path);
}
