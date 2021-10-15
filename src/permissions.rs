use std::path::PathBuf;
use std::process::Command;

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
