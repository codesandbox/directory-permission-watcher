use std::path::Path;
use std::process::Command;

pub fn update_permission_recursive(path: &Path) {
    let path_str = path.as_os_str();
    let cmd_output = Command::new("chmod")
        .args(["-R", "g+rw"])
        .arg(path_str)
        .output();

    match cmd_output {
        Ok(_) => {
            println!("Updated permissions of {:?}", path_str);
        }
        Err(err) => {
            println!("Could not update permissions of {:?}, {:?}", path_str, err);
        }
    }
}
