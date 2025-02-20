use colored::Colorize;
use extract::extract_package;
use fs_extra::dir::{copy, CopyOptions};
use getpkg::getpkg;
use mk_symlink::create_symlinks;
use search::searchpkg;
use std::{
    env::args,
    fs::{self, File},
    io::Read,
    path::Path,
    process::exit,
};

pub mod extract;
pub mod getpkg;
pub mod mk_symlink;
pub mod ndraey_dm_custom;
pub mod parse_pkg_index;
pub mod search;

#[allow(unused)]
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<String> = args().collect();
    let args = &args[1..];
    let cmd = &args[0];
    let mut search_terms: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();

    match cmd.as_str() {
        "__dbg_pkg__" => {
            if let Ok(mut pkgif) = File::open("gobbled.gob") {
                let mut index = String::new();
                if pkgif.read_to_string(&mut index).is_ok() {
                    let pkg_index = parse_pkg_index::ppkgi(&index);
                    for pkg in &pkg_index {
                        println!("~> {:?}\n", pkg);
                    }
                } else {
                    eprintln!("└─ Unable to read package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
                }
            } else {
                eprintln!("└─ Unable to open package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
            }
        }
        "info" | "about" => {
            if let Ok(mut pkgif) = File::open("gobbled.gob") {
                let mut index = String::new();
                if pkgif.read_to_string(&mut index).is_ok() {
                    let pkg_index = parse_pkg_index::ppkgi(&index);
                    println!("{}","┌[About packages]".green().bold());
                    for pkg in &pkg_index {
                        if search_terms.iter().any(|term| pkg.0.contains(term)) {
                            println!("{}","├─[Package Info]".green().bold());
                            println!("├─ Name:          {}", pkg.0.green().bold());
                            println!("├─ Version:       {}", pkg.1.version.green().bold());
                            println!("├─ Description:   {}", pkg.1.description.green().bold());
                            println!("├─ Supports GUI:  {}", if pkg.1.gui { "Yes".green().bold() } else { "No".red().bold() });
                            println!("├─ Binary Located at: {}", pkg.1.binary_at.join("\n\t\t      ").green().bold());
                            println!("├─ Symlink it creates: {}", pkg.1.symlink_names.join(", ").green().bold());
                            println!("├─ Icon URL:      {}", pkg.1.icon_at.green().bold());
                        }
                    }
                    println!("{}","└[DONE!]".green().bold());
                    let not_found: Vec<_> = search_terms.iter().filter(|term| !pkg_index.iter().any(|pkg| pkg.0.contains(*term))).collect();
                    if !not_found.is_empty() {
                        println!("{}", "┌[Following packages were not found!]".red().bold());
                        for term in not_found {
                            println!("├─ {}", term.red().bold());
                        }
                        println!("{}", "└[Please look online for correct package names or contact us!]".red().bold());
                    }
                } else {
                    eprintln!("└─ Unable to read package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
                }
            } else {
                eprintln!("└─ Unable to open package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
                
            }
        }
        "search" | "look" | "find" => {
            if let Ok(mut pkgif) = File::open("gobbled.gob") {
                let mut index = String::new();
                if pkgif.read_to_string(&mut index).is_ok() {
                    let pkg_index = parse_pkg_index::ppkgi(&index);
                    println!("{}", "┌[Found packages]".green().bold());
                    for pkg in &pkg_index {
                        if search_terms.iter().any(|term| pkg.0.contains(term)) {
                            println!("├─ {}", pkg.0.green().bold());
                        }
                    }
                    println!("{}", "└[DONE!]".green().bold());
                    let not_found: Vec<_> = search_terms.iter().filter(|term| !pkg_index.iter().any(|pkg| pkg.0.contains(*term))).collect();
                    if !not_found.is_empty() {
                        println!("{}", "┌[Following packages were not found!]".red().bold());
                        for term in not_found {
                            println!("├─ {}", term.red().bold());
                        }
                        println!("{}", "└[Please look online for correct package names or contact us!]".red().bold());
                    }
                } else {
                    eprintln!("└─ Unable to read package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
                }
            } else {
                eprintln!("└─ Unable to open package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
            }
        }
        "local-install" => {}
        "install" => {
            println!("{}", "Looking if all packages exists in the index...".green().bold());
            let mut pkg_index = searchpkg(&search_terms, &getpkgindex());
            if !ifroot() {
                eprintln!("{}", "You need to be root to install packages!".red().bold());
                exit(1);
            }
            for pkg in search_terms {
                let mut pkginfo = pkg_index.get_mut(&pkg).unwrap();
                println!("{}{}", "Downloading package : ".green().bold(), pkginfo.name);
                match getpkg(pkginfo).await {
                    Ok(pkg_p) => {
                        if pkginfo.extractable {
                            println!("{}", "Extracting package...".green().bold());
                            let extract_dir = format!("/tmp/{}_{}", pkginfo.name, pkginfo.version);
                            match extract_package(&pkg_p, &extract_dir) {
                                Ok(_) => {
                                    let pkg_edir_path = Path::new(&extract_dir);
                                    let entries: Vec<_> = fs::read_dir(pkg_edir_path)
                                        .unwrap()
                                        .filter_map(|e| e.ok())
                                        .collect();
                                    let source = if entries.len() == 1 && entries[0].path().is_dir() {
                                        entries[0].path()
                                    } else {
                                        pkg_edir_path.to_path_buf()
                                    };
                                    let bin_dir = format!("{}/gobpkg_{}", "/usr/bin", &pkginfo.name);
                                    if fs::create_dir_all(&bin_dir).is_ok() {
                                        let mut options = CopyOptions::new();
                                        options.overwrite = true;
                                        options.copy_inside = true;
                                        match copy(&source, &bin_dir, &options) {
                                            Ok(_) => {
                                                println!("{}", "Creating symlinks...".green().bold());
                                                let slinks = create_symlinks(&bin_dir, pkginfo);
                                                println!("{}", "Cleaning up files...".green().bold());
                                                if fs::remove_file(&pkg_p).is_ok() {
                                                    if fs::remove_dir_all(&extract_dir).is_ok() {
                                                        println!("{}", "Package installed successfully!".green().bold());
                                                        if pkginfo.gui {
                                                            println!("{}", "Integrating with desktop...".green().bold());
                                                            let desktop_file = format!("/usr/share/applications/{}.desktop", pkginfo.name);
                                                            async fn download_icon(url: &str, local_path: &str) -> Result<(), Box<dyn std::error::Error>> {
                                                                let response = reqwest::get(url).await?;
                                                                let bytes = response.bytes().await?;
                                                                fs::write(local_path, &bytes)?;
                                                                Ok(())
                                                            }
                                                            let icon_local = format!("{}/{}_icon.png", bin_dir, pkginfo.name);
                                                            match download_icon(&pkginfo.icon_at, &icon_local).await {
                                                                Ok(_) => println!("{}", "Icon downloaded successfully.".green().bold()),
                                                                Err(e) => eprintln!("{}: {}", "Failed to download icon".red().bold(), e),
                                                            }
                                                            //println!("symlinks: {:?}", slinks);
                                                            let desktop_file_content = format!(
                                                                "[Desktop Entry]\nVersion=1.0\nType=Application\nName={}\nExec={}\nIcon={}\nTerminal=false\nCategories=Utility;",
                                                                pkginfo.name,
                                                                slinks.join(" "),
                                                                icon_local
                                                            );
                                                            
                                                            match fs::write(&desktop_file, &desktop_file_content) {
                                                                Ok(_) => {
                                                                   // println!("{}", "Desktop entry created successfully!".green().bold());
                                                                    match tokio::process::Command::new("update-desktop-database")
                                                                        .arg("/usr/share/applications")
                                                                        .status()
                                                                        .await
                                                                    {
                                                                        Ok(status) if status.success() => {
                                                                            println!("{}", "Desktop database updated successfully.".green().bold());
                                                                        }
                                                                        Ok(status) => {
                                                                            eprintln!("{}: exit code {}", "Failed to update desktop database".red().bold(), status);
                                                                        }
                                                                        Err(e) => {
                                                                            eprintln!("{}: {}", "Error updating desktop database".red().bold(), e);
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    eprintln!("{}", "Unable to create desktop entry!".red().bold());
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        eprintln!("{}", "Unable to remove extracted package directory!".red().bold());
                                                    }
                                                } else {
                                                    eprintln!("{}", "Unable to remove downloaded package file!".red().bold());
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("{}: {}", "Unable to copy package from extracted source".red().bold(), e);
                                                exit(1);
                                            }
                                        }
                                    } else {
                                        eprintln!("{}", "Unable to create binary directory!".red().bold());
                                        exit(1);
                                    }
                                }
                                Err(_) => {
                                    eprintln!("{}", "Unable to extract package!".red().bold());
                                    exit(1);
                                }
                            }
                        } else {
                            println!("{}", "Package is not extractable, skipping extraction...".green().bold());
                            let bin_dir = format!("{}/gobpkg_{}", "/usr/bin", &pkginfo.name);
                            if fs::create_dir_all(&bin_dir).is_ok() {
                                let target_file = format!(
                                    "{}/{}",
                                    &bin_dir,
                                    Path::new(&pkg_p).file_name().unwrap().to_string_lossy()
                                );
                                match fs::copy(&pkg_p, &target_file) {
                                    Ok(_) => {
                                        println!("{}", "Creating symlinks...".green().bold());
                                        let slinks = create_symlinks(&bin_dir, pkginfo);
                                        println!("{}", "Cleaning up files...".green().bold());
                                        println!("gui? : {}", pkginfo.gui);
                                        if fs::remove_file(&pkg_p).is_ok() {
                                            if pkginfo.gui {
                                                println!("{}", "Integrating with desktop...".green().bold());
                                                async fn download_icon(url: &str, local_path: &str) -> Result<(), Box<dyn std::error::Error>> {
                                                    let response = reqwest::get(url).await?;
                                                    let bytes = response.bytes().await?;
                                                    fs::write(local_path, &bytes)?;
                                                    Ok(())
                                                }
                                                println!("symlinks: {:?}", slinks);
                                                let host = match sys_info::hostname() {
                                                    Ok(host) => host,
                                                    Err(_) => {
                                                        eprintln!("{}","Error getting hostname".red().bold());
                                                        exit(1);
                                                    }
                                                };
                                                let desktop_file = format!("/home/{}/.local/share/applications/{}.desktop", host, pkginfo.name);
                                                println!("host: {}", host);
                                                let icon_dir = format!("/home/{}/{}", host, ".local/gobicons");
                                                if !Path::new(&icon_dir).exists() {
                                                    if let Err(e) = fs::create_dir_all(&icon_dir) {
                                                        eprintln!("{}: {}", "Failed to create icon directory".red().bold(), e);
                                                        exit(1);
                                                    }
                                                }
                                                let icon_local = format!("{}/{}_icon.png", icon_dir, pkginfo.name);
                                                match download_icon(&pkginfo.icon_at, &icon_local).await {
                                                    Ok(_) => println!("{}", "Icon downloaded successfully.".green().bold()),
                                                    Err(e) => eprintln!("{}: {}", "Failed to download icon".red().bold(), e),
                                                }
                                                let desktop_file_content = format!(
                                                    "[Desktop Entry]\nVersion=1.0\nType=Application\nName={}\nExec={}\nIcon={}\nTerminal=true\nCategories=Accessories;",
                                                    pkginfo.name,
                                                    slinks.join(" "),
                                                    icon_local
                                                );
                                                println!("{}", desktop_file_content);
                                                match fs::write(&desktop_file, &desktop_file_content) {
                                                    Ok(_) => {
                                                        println!("{}", "Desktop entry created successfully!".green().bold());
                                                        match tokio::process::Command::new("update-desktop-database")
                                                            .arg("/usr/share/applications")
                                                            .status()
                                                            .await
                                                        {
                                                            Ok(status) if status.success() => {
                                                                println!("{}", "Desktop database updated successfully.".green().bold());
                                                            }
                                                            Ok(status) => {
                                                                eprintln!("{}: exit code {}", "Failed to update desktop database".red().bold(), status);
                                                            }
                                                            Err(e) => {
                                                                eprintln!("{}: {}", "Error updating desktop database".red().bold(), e);
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        eprintln!("{}", "Unable to create desktop entry!".red().bold());
                                                    }
                                                }
                                            }
                                            println!("{}", "Package installed successfully!".green().bold());
                                        } else {
                                            eprintln!("{}", "Unable to remove downloaded package file!".red().bold());
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("{}: {}", "Unable to copy package file!".red().bold(), e);
                                        exit(1);
                                    }
                                }
                            } else {
                                eprintln!("{}", "Unable to create binary directory!".red().bold());
                                exit(1);
                            }
                        }
                    }
                    Err(_) => {
                        eprintln!("{}", "Unable to fetch package!".red().bold());
                        exit(1);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn getpkgindex() -> String {
    match File::open("gobbled.gob") {
        Ok(mut pkgf) => {
            let mut index = String::new();
            if pkgf.read_to_string(&mut index).is_ok() {
                index
            } else {
                eprintln!("{}", "Unable to read package index file! Try fetching it first with \"gob fetch\" or \"gob update\".".red().bold());
                exit(1);
            }
        }
        Err(_) => {
            eprintln!("{}", "Unable to open package index file! Try fetching it first with \"gob fetch\" or \"gob update\".".red().bold());
            exit(1);
        }
    }
}

pub fn ifroot() -> bool {
    unsafe { libc::geteuid() == 0 }
}
