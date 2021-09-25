use chownr::chownr;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use nix::unistd::{Gid, Uid};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    })?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P, owners: (u32, u32)) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if !event.kind.is_remove() {
                    println!("watch error: {:?}", event.kind);
                    check_permissions(event.clone().paths, owners);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn check_permissions(paths: Vec<PathBuf>, parent_owners: (u32, u32)) {
    for path in paths.clone().iter() {
        let p = path.as_path();

        if cfg!(debug_assertions) {
            println!("Validating file permissions for {:?}", p);
        }

        match get_permissions(p) {
            Ok((curr_uid, curr_gid)) => {
                let (parent_uid, parent_gid) = parent_owners;

                if parent_uid != curr_uid && parent_gid != curr_gid {
                    match chownr(
                        p,
                        Some(Uid::from_raw(parent_uid)),
                        Some(Gid::from_raw(parent_gid)),
                    ) {
                        Ok(()) => println!("Updated file owners"),
                        Err(err) => println!("Could not update file owners {:?}", err),
                    };
                } else if cfg!(debug_assertions) {
                    println!("File permissions of {:?} are in order", p);
                }
            }
            Err(err) => println!("Could not load file data {:?}, {:?}", p, err),
        };
    }
}

fn get_permissions(path: &Path) -> io::Result<(u32, u32)> {
    let metadata = fs::metadata(path)?;
    return Ok((metadata.uid(), metadata.gid()));
}

fn start_watcher(path: &str, owners: (u32, u32)) {
    println!("watching: {}", path);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(path, owners).await {
            println!("error: {:?}", e)
        }
    });
}

fn main() {
    let path = &std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    match get_permissions(Path::new(path)) {
        Ok(owners) => {
            start_watcher(path, owners);
        }
        Err(e) => panic!("fs error: {:?}", e),
    }
}
