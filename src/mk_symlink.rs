use colored::Colorize;
use std::fs::{self, remove_file};
use std::os::unix::fs::{symlink, PermissionsExt};

pub fn create_symlinks(
    bin_dir: &str,
    pkginfo: &mut crate::parse_pkg_index::Package,
) -> Vec<String> {
    let mut paths = Vec::new();
    let host = match sys_info::hostname() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{}: {}", "Failed to get hostname".red().bold(), e);
            "root".to_string()
        }
    };
    let gob_dir = format!("/home/{}/.gob", host);

    if pkginfo.extractable {
        for (i, bin_entry) in pkginfo.binary_at.iter().enumerate() {
            let slink_name = pkginfo.symlink_names.get(i).unwrap_or(&pkginfo.name);
            let binary_path = format!("{}/{}/{}", bin_dir, pkginfo.name, bin_entry);
            let symlink_path = format!("{}/{}", gob_dir, slink_name);

            if fs::symlink_metadata(&symlink_path).is_ok() {
                if let Err(e) = remove_file(&symlink_path) {
                    eprintln!("{}: {}: {}", "Failed to remove existing symlink".red().bold(), symlink_path, e);
                    continue;
                }
            }
            if let Err(e) = symlink(&binary_path, &symlink_path) {
                eprintln!("{}: {}: {}", "Failed to create symlink".red().bold(), symlink_path, e);
                continue;
            }
            if let Ok(metadata) = fs::metadata(&binary_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(permissions.mode() | 0o111);
                if let Err(e) = fs::set_permissions(&binary_path, permissions.clone()) {
                    eprintln!("{}: {}", "Failed to set executable permission on binary".red().bold(), e);
                }
                if let Err(e) = fs::set_permissions(&symlink_path, permissions) {
                    eprintln!("{}: {}", "Failed to set executable permission on symlink".red().bold(), e);
                }
            } else {
                eprintln!("{}: {}", "Failed to get metadata for binary".red().bold(), binary_path);
            }
            paths.push(symlink_path);
        }
    } else {
        let binary_path = format!("{}",bin_dir);
        if let Ok(metadata) = fs::metadata(&binary_path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | 0o111);
            if let Err(e) = fs::set_permissions(&binary_path, permissions) {
                eprintln!("{}: {}", "Failed to set executable permission on binary".red().bold(), e);
            }
            let mut perms = fs::metadata(format!("{}_{}",binary_path,pkginfo.name)).unwrap().permissions();
            perms.set_mode(perms.mode() | 0o111);
        } else {
            eprintln!("{}: {}", "Failed to get metadata for binary".red().bold(), binary_path);
        }

        paths.push(binary_path);
    }
    paths
}
