use colored::Colorize;
use std::fs::{self, remove_file};
use std::os::unix::fs::{symlink, PermissionsExt};

pub fn create_symlinks(
    bin_dir: &str,
    pkginfo: &mut crate::parse_pkg_index::Package,
) -> Vec<String> {
    let mut symlinks: Vec<String> = Vec::new();

    if pkginfo.gui && pkginfo.symlink_names.len() > 1 {
        eprintln!(
            "{}",
            "More than 1 symlink file detected, turning off desktop integration"
                .red()
                .bold()
        );
        pkginfo.gui = false;
    }

    for (i, binf) in pkginfo.binary_at.iter().enumerate() {
        let slinkn = pkginfo.symlink_names.get(i).unwrap_or(&pkginfo.name);

        let binf = if pkginfo.extractable {
            format!("{}/{}", bin_dir, binf)
        } else {
            format!("{}/{}_{}", bin_dir, &pkginfo.name, &pkginfo.name)
        };

        let slink = format!("/usr/bin/{}", slinkn);

        // Check if symlink exists and remove it
        if fs::symlink_metadata(&slink).is_ok() {
            if let Err(e) = remove_file(&slink) {
                eprintln!("{}: {}: {}", "Failed to remove existing symlink".red().bold(), slink, e);
                continue;
            }
        }

        match symlink(&binf, &slink) {
            Ok(_) => {
                // Make the target binary executable
                if let Ok(metadata) = fs::metadata(&binf) {
                    let mut permissions = metadata.permissions();
                    permissions.set_mode(permissions.mode() | 0o111); // Add executable bits
                    if let Err(e) = fs::set_permissions(&binf, permissions.clone()) {
                        eprintln!("{}: {}", "Failed to set executable permission".red().bold(), e);
                    }
                    if let Err(e) = fs::set_permissions(&slink, permissions) {
                        eprintln!("{}: {}", "Failed to set executable permission".red().bold(), e);
                    }
                } else {
                    eprintln!("{}: {}", "Failed to get metadata for binary".red().bold(), binf);
                }
                symlinks.push(slink);
            }
            Err(e) => eprintln!("{}: {}: {}", "Failed to create symlink".red().bold(), slink, e),
        }
    }

    symlinks
}
