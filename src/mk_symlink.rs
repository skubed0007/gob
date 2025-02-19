use colored::Colorize;
use rayon::prelude::*;
use std::fs::{self, Permissions};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{symlink, PermissionsExt};

pub fn create_symlinks(
    bin_dir: &str,
    pkginfo: &mut crate::parse_pkg_index::Package,
) -> Vec<String> {
    if !pkginfo.extractable {
        println!("{}", "creating symlinks...".green().bold());

        // Debug: Print initial package information
        println!(
            "{}",
            format!("DEBUG: pkginfo.name = {}", pkginfo.name).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: pkginfo.gui = {}", pkginfo.gui).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: binary_at = {:?}", pkginfo.binary_at).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: symlink_names = {:?}", pkginfo.symlink_names).magenta()
        );

        if pkginfo.gui && pkginfo.symlink_names.len() > 1 {
            eprintln!(
                "! {} {}",
                "Warning:".yellow().bold(),
                "Multiple symlinks requested for GUI package, setting GUI to false".yellow()
            );
            pkginfo.gui = false;
        }

        let created_symlinks: Vec<String> = pkginfo
            .binary_at
            .par_iter()
            .enumerate()
            .map(|(i, bin_rel)| {
                println!(
                    "{}",
                    format!(
                        "DEBUG: Processing index {} with binary_rel '{}'",
                        i, bin_rel
                    )
                    .bright_blue()
                );

                // Construct source path
                let src = format!("{}/{}_{}", bin_dir, pkginfo.name, pkginfo.name);
                println!("{}", format!("DEBUG: Computed source path: {}", src).cyan());

                // Set the source file executable
                println!(
                    "{}",
                    format!("DEBUG: Checking existence of source file {}", src).cyan()
                );
                match fs::metadata(&src) {
                    Ok(metadata) => {
                        println!(
                            "{}",
                            format!(
                                "DEBUG: Source file {} exists (mode: {:o})",
                                src,
                                metadata.mode()
                            )
                            .cyan()
                        );
                        println!(
                            "{}",
                            format!("DEBUG: Setting permissions to 755 for source file {}", src)
                                .cyan()
                        );
                        if let Err(e) = fs::set_permissions(&src, Permissions::from_mode(0o755)) {
                            eprintln!(
                                "└─ Failed to set permissions for source file {}: {}",
                                src.red().bold(),
                                e
                            );
                        } else {
                            println!(
                                "{}",
                                format!("DEBUG: Source file {} is now executable.", src).blue()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "└─ ERROR: Source file {} does not exist: {}",
                            src.red().bold(),
                            e
                        );
                    }
                }

                // Determine symlink name
                let symlink_name = pkginfo
                    .symlink_names
                    .get(i)
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        pkginfo
                            .binary_at
                            .get(i)
                            .and_then(|s| s.rsplit('/').next())
                            .unwrap_or(pkginfo.name.as_str())
                    });
                println!(
                    "{}",
                    format!("DEBUG: Using symlink name: {}", symlink_name).cyan()
                );

                // Construct destination path
                let dst = format!("/usr/bin/{}", symlink_name);
                println!(
                    "{}",
                    format!("DEBUG: Destination path computed as: {}", dst).cyan()
                );

                // Attempt to remove any existing file at destination
                println!(
                    "{}",
                    format!("DEBUG: Removing any existing file at {}", dst).cyan()
                );
                match fs::remove_file(&dst) {
                    Ok(_) => println!(
                        "{}",
                        format!("DEBUG: Successfully removed existing file at {}", dst).cyan()
                    ),
                    Err(e) => println!(
                        "{}",
                        format!("DEBUG: No file removed at {}: {}", dst, e).cyan()
                    ),
                }

                // Create the symlink
                println!(
                    "{}",
                    format!("DEBUG: Creating symlink from {} to {}", src, dst).cyan()
                );
                match symlink(&src, &dst) {
                    Err(e) => {
                        eprintln!("└─ Failed to create symlink {}: {}", dst.red().bold(), e);
                    }
                    Ok(_) => {
                        println!("{}", format!("DEBUG: Symlink created at {}.", dst).blue());
                        // Check metadata: if it's a symlink, skip permission changes.
                        match fs::symlink_metadata(&dst) {
                            Ok(metadata) if metadata.file_type().is_symlink() => {
                                println!(
                                    "{}",
                                    format!(
                                        "DEBUG: {} is a symlink, permission change skipped.",
                                        dst
                                    )
                                    .blue()
                                );
                            }
                            Ok(_) => {
                                println!(
                                    "{}",
                                    format!("DEBUG: Setting permissions to 755 for {}", dst).blue()
                                );
                                if let Err(e) =
                                    fs::set_permissions(&dst, Permissions::from_mode(0o755))
                                {
                                    eprintln!(
                                        "└─ Failed to set permissions for {}: {}",
                                        dst.red().bold(),
                                        e
                                    );
                                } else {
                                    println!(
                                        "{}",
                                        format!("DEBUG: Permissions successfully set for {}.", dst)
                                            .blue()
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "└─ Failed to get metadata for {}: {}",
                                    dst.red().bold(),
                                    e
                                );
                            }
                        }
                    }
                }

                println!(
                    "{}",
                    format!("DEBUG: Finished processing index {}. Symlink at {}", i, dst).green()
                );
                dst
            })
            .collect();

        println!("{}", "DEBUG: All symlinks processed.".green().bold());
        created_symlinks
    } else {
        println!("{}", "creating symlinks...".green().bold());

        // Debug: Print initial package information
        println!(
            "{}",
            format!("DEBUG: pkginfo.name = {}", pkginfo.name).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: pkginfo.gui = {}", pkginfo.gui).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: binary_at = {:?}", pkginfo.binary_at).magenta()
        );
        println!(
            "{}",
            format!("DEBUG: symlink_names = {:?}", pkginfo.symlink_names).magenta()
        );

        if pkginfo.gui && pkginfo.symlink_names.len() > 1 {
            eprintln!(
                "! {} {}",
                "Warning:".yellow().bold(),
                "Multiple symlinks requested for GUI package, setting GUI to false".yellow()
            );
            pkginfo.gui = false;
        }

        let created_symlinks: Vec<String> = pkginfo
            .binary_at
            .par_iter()
            .enumerate()
            .map(|(i, bin_rel)| {
                println!(
                    "{}",
                    format!(
                        "DEBUG: Processing index {} with binary_rel '{}'",
                        i, bin_rel
                    )
                    .bright_blue()
                );

                // Construct source path
                let src = format!("{}/{}/{}", bin_dir, pkginfo.name, pkginfo.name);
                println!("{}", format!("DEBUG: Computed source path: {}", src).cyan());

                // Set the source file executable
                println!(
                    "{}",
                    format!("DEBUG: Checking existence of source file {}", src).cyan()
                );
                match fs::metadata(&src) {
                    Ok(metadata) => {
                        println!(
                            "{}",
                            format!(
                                "DEBUG: Source file {} exists (mode: {:o})",
                                src,
                                metadata.mode()
                            )
                            .cyan()
                        );
                        println!(
                            "{}",
                            format!("DEBUG: Setting permissions to 755 for source file {}", src)
                                .cyan()
                        );
                        if let Err(e) = fs::set_permissions(&src, Permissions::from_mode(0o755)) {
                            eprintln!(
                                "└─ Failed to set permissions for source file {}: {}",
                                src.red().bold(),
                                e
                            );
                        } else {
                            println!(
                                "{}",
                                format!("DEBUG: Source file {} is now executable.", src).blue()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "└─ ERROR: Source file {} does not exist: {}",
                            src.red().bold(),
                            e
                        );
                    }
                }

                // Determine symlink name
                let symlink_name = pkginfo
                    .symlink_names
                    .get(i)
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        pkginfo
                            .binary_at
                            .get(i)
                            .and_then(|s| s.rsplit('/').next())
                            .unwrap_or(pkginfo.name.as_str())
                    });
                println!(
                    "{}",
                    format!("DEBUG: Using symlink name: {}", symlink_name).cyan()
                );

                // Construct destination path
                let dst = format!("/usr/bin/{}", symlink_name);
                println!(
                    "{}",
                    format!("DEBUG: Destination path computed as: {}", dst).cyan()
                );

                // Attempt to remove any existing file at destination
                println!(
                    "{}",
                    format!("DEBUG: Removing any existing file at {}", dst).cyan()
                );
                match fs::remove_file(&dst) {
                    Ok(_) => println!(
                        "{}",
                        format!("DEBUG: Successfully removed existing file at {}", dst).cyan()
                    ),
                    Err(e) => println!(
                        "{}",
                        format!("DEBUG: No file removed at {}: {}", dst, e).cyan()
                    ),
                }

                // Create the symlink
                println!(
                    "{}",
                    format!("DEBUG: Creating symlink from {} to {}", src, dst).cyan()
                );
                match symlink(&src, &dst) {
                    Err(e) => {
                        eprintln!("└─ Failed to create symlink {}: {}", dst.red().bold(), e);
                    }
                    Ok(_) => {
                        println!("{}", format!("DEBUG: Symlink created at {}.", dst).blue());
                        // Check metadata: if it's a symlink, skip permission changes.
                        match fs::symlink_metadata(&dst) {
                            Ok(metadata) if metadata.file_type().is_symlink() => {
                                println!(
                                    "{}",
                                    format!(
                                        "DEBUG: {} is a symlink, permission change skipped.",
                                        dst
                                    )
                                    .blue()
                                );
                            }
                            Ok(_) => {
                                println!(
                                    "{}",
                                    format!("DEBUG: Setting permissions to 755 for {}", dst).blue()
                                );
                                if let Err(e) =
                                    fs::set_permissions(&dst, Permissions::from_mode(0o755))
                                {
                                    eprintln!(
                                        "└─ Failed to set permissions for {}: {}",
                                        dst.red().bold(),
                                        e
                                    );
                                } else {
                                    println!(
                                        "{}",
                                        format!("DEBUG: Permissions successfully set for {}.", dst)
                                            .blue()
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "└─ Failed to get metadata for {}: {}",
                                    dst.red().bold(),
                                    e
                                );
                            }
                        }
                    }
                }

                println!(
                    "{}",
                    format!("DEBUG: Finished processing index {}. Symlink at {}", i, dst).green()
                );
                dst
            })
            .collect();

        println!("{}", "DEBUG: All symlinks processed.".green().bold());
        created_symlinks
    }
}
