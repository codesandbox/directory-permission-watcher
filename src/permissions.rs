use std::path::PathBuf;
use std::process::Command;

use crate::common_path::common_path_all;

pub fn check_permission_recursive(path: PathBuf) {
    if let Ok(path_str) = path.clone().into_os_string().into_string() {
        let cmd_output = Command::new("chmod")
            .args(["-R", "g+rw", path_str.as_str()])
            .output();

        match cmd_output {
            Ok(_) => {
                println!("Updated permissions of {:?}", path.as_os_str());
            }
            Err(err) => {
                println!(
                    "Could not update permissions of {:?}, {:?}",
                    path.as_os_str(),
                    err
                );
            }
        }
    }
}

pub fn check_permissions(paths: Vec<PathBuf>) {
    if let Some(common_path) = common_path_all(paths.clone()) {
        check_permission_recursive(common_path);
    } else {
        println!("Could not find common path for {:?}", paths);
    }
}
