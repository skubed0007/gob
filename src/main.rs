use colored::Colorize;
use extract::extract_package;
use fs_extra::dir::{copy, CopyOptions};
use getpkg::getpkg;
use mk_symlink::create_symlinks;
use search::searchpkg;
use std::{
    env::{self, args},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::exit,
};

pub mod extract;
pub mod getpkg;
pub mod mk_symlink;
pub mod ndraey_dm_custom;
pub mod parse_pkg_index;
pub mod search;


pub fn help() {
    println!("{}", "┌[Gob Package Manager - Help]".white().bold());
    println!("{}", "├─ Usage: gob <command> [options]".white());
    println!("{}", "├─ Commands:".white().bold());
    println!(
        "{} {}",
        "│   ├─ update".green().bold(),
        "- Update the package index (fetches the latest index) along with packages themselves".white()
    );
    println!(
        "{} {}",
        "│   ├─ search <term>".green().bold(),
        "- Search for packages".white()
    );
    println!(
        "{} {}",
        "│   ├─ info <package>".green().bold(),
        "- Display package information".white()
    );
    println!(
        "{} {}",
        "│   ├─ install <package>".green().bold(),
        "- Install a package".white()
    );
    println!(
        "{} {}",
        "│   └─ local-install".green().bold(),
        "- Install package locally".white()
    );
    println!("{}", "└[For more info, try 'gob help <command>']".white().bold());
}


#[allow(unused)]
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 1{
        help();
        exit(1);
    }
    let args = &args[1..];
    let cmd = &args[0];
    let mut search_terms: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();
    let host = match sys_info::hostname() {
        Ok(host) => host,
        Err(_) => {
            eprintln!("{}", "Error getting hostname".red().bold());
            exit(1);
        }
    };

    let indexfl =
        if let Some(indexfile_arg) = args.iter().position(|arg| arg.starts_with("indexfile=")) {
            let indexfile = args[indexfile_arg]
                .trim_start_matches("indexfile=")
                .to_string();
            if indexfile_arg > 0 {
                search_terms.remove(indexfile_arg - 1);
            }
            indexfile
        } else {
            format!("/home/{}/.gobbled.gob", host)
        };
    let installdir = format!("/home/{}/.gob", host);
    if !Path::new(&installdir).exists() {
        match fs::create_dir(&installdir) {
            Ok(_) => println!(
                "{}: {}",
                "Created install directory".green().bold(),
                installdir
            ),
            Err(e) => {
                eprintln!(
                    "{}: {}",
                    "Failed to create install directory".red().bold(),
                    e
                );
                exit(1);
            }
        }
    }
    fn add_gob_to_path_for_host(host: &str) {
        let path_var = env::var("PATH").unwrap_or_default();
        let gob_path = format!("/home/{}/.gob", host);
    
        if !path_var.split(':').any(|p| p == gob_path) {
            println!("you need to restart your terminal as gob just got added to your awesome shell's path");
            let new_path = format!("{}:{}", gob_path, path_var);
            env::set_var("PATH", &new_path);
    
            if let Ok(home) = env::var("HOME") {
                let shell_configs = vec![
                    (
                        format!("{}/.bashrc", home),
                        format!("\nexport PATH=\"{}:$PATH\"\n", gob_path),
                    ),
                    (
                        format!("{}/.zshrc", home),
                        format!("\nexport PATH=\"{}:$PATH\"\n", gob_path),
                    ),
                    (
                        format!("{}/.config/fish/config.fish", home),
                        format!("\nset -x PATH {} $PATH\n", gob_path),
                    ),
                ];
    
                for (config_path, export_line) in shell_configs {
                    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&config_path) {
                        let _ = file.write_all(export_line.as_bytes());
                    }
                }
            }
        }
    }
    
    
    add_gob_to_path_for_host(&host);

    match cmd.as_str() {
        "help" | "h" => {
            help();
            exit(0);
        }
        "update" => {
            println!(
                "{}{}",
                "Trying to download package index from : ".green(),
                "https://raw.githubusercontent.com/skubed0007/gob/main/gobbled.gob"
                    .red()
                    .bold()
            );
            let index_url = "https://raw.githubusercontent.com/skubed0007/gob/main/gobbled.gob";
            let index = match reqwest::get(index_url).await {
                Ok(res) => {
                    let index = res.text().await.unwrap();
                    index
                }
                Err(e) => {
                    eprintln!("{}: {}", "Failed to fetch package index".red().bold(), e);
                    exit(1);
                }
            };
            match fs::write(&indexfl, &index) {
                Ok(_) => println!("{}", "Package index updated successfully!".green().bold()),
                Err(e) => eprintln!("{}: {}", "Failed to write package index".red().bold(), e),
            }
        }
        "__dbg_pkg__" => {
            if let Ok(mut pkgif) = File::open(indexfl) {
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
            if let Ok(mut pkgif) = File::open(indexfl) {
                let mut index = String::new();
                if pkgif.read_to_string(&mut index).is_ok() {
                    let pkg_index = parse_pkg_index::ppkgi(&index);
                    println!("{}", "┌[About packages]".green().bold());
                    for pkg in &pkg_index {
                        if search_terms.iter().any(|term| pkg.0.contains(term)) {
                            println!("{}", "├─[Package Info]".green().bold());
                            println!("├─ Name:          {}", pkg.0.green().bold());
                            println!("├─ Version:       {}", pkg.1.version.green().bold());
                            println!("├─ Description:   {}", pkg.1.description.green().bold());
                            println!(
                                "├─ Supports GUI:  {}",
                                if pkg.1.gui {
                                    "Yes".green().bold()
                                } else {
                                    "No".red().bold()
                                }
                            );
                            println!(
                                "├─ Binary Located at: {}",
                                pkg.1.binary_at.join("\n\t\t      ").green().bold()
                            );
                            println!(
                                "├─ Symlink it creates: {}",
                                pkg.1.symlink_names.join(", ").green().bold()
                            );
                            println!("├─ Icon URL:      {}", pkg.1.icon_at.green().bold());
                        }
                    }
                    println!("{}", "└[DONE!]".green().bold());
                    let not_found: Vec<_> = search_terms
                        .iter()
                        .filter(|term| !pkg_index.iter().any(|pkg| pkg.0.contains(*term)))
                        .collect();
                    if !not_found.is_empty() {
                        println!("{}", "┌[Following packages were not found!]".red().bold());
                        for term in not_found {
                            println!("├─ {}", term.red().bold());
                        }
                        println!(
                            "{}",
                            "└[Please look online for correct package names or contact us!]"
                                .red()
                                .bold()
                        );
                    }
                } else {
                    eprintln!("└─ Unable to read package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
                }
            } else {
                eprintln!("└─ Unable to open package index file! Try fetching it first with \"gob fetch\" or \"gob update\". {}", "Error".red().bold());
            }
        }
        "search" | "look" | "find" => {
            if let Ok(mut pkgif) = File::open(indexfl) {
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
                    let not_found: Vec<_> = search_terms
                        .iter()
                        .filter(|term| !pkg_index.iter().any(|pkg| pkg.0.contains(*term)))
                        .collect();
                    if !not_found.is_empty() {
                        println!("{}", "┌[Following packages were not found!]".red().bold());
                        for term in not_found {
                            println!("├─ {}", term.red().bold());
                        }
                        println!(
                            "{}",
                            "└[Please look online for correct package names or contact us!]"
                                .red()
                                .bold()
                        );
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
            match fs::read_to_string(indexfl) {
                Ok(pkgindex_str) => {
                    let mut pkgindex = parse_pkg_index::ppkgi(&pkgindex_str);
                    searchpkg(&search_terms, &pkgindex_str);
                    for pkg in &mut pkgindex {
                        if search_terms.iter().any(|term| &pkg.0 == &term) {
                            //check if package is extractable and thus create proper install dir and links
                            println!("{}: {}", "Installing package".green().bold(), &pkg.0);
                            let binfolder_file = {
                                if pkg.1.extractable {
                                    format!("{}/{}", installdir, &pkg.0)
                                } else {
                                    format!("{}/{}", installdir, &pkg.1.name)
                                }
                            };
                            match getpkg(&pkg.1).await {
                                Ok(tmpfp) => {
                                    if pkg.1.extractable {
                                        // Construct the extraction directory path
                                        let extractdir =
                                            format!("/home/{}/.gob/{}", host, &pkg.1.name);

                                        // Attempt to extract the package from tmpfp into extractdir
                                        match extract_package(&tmpfp, &extractdir) {
                                            Ok(_) => {

                                                // Create the final package installation directory path
                                                let pkgidir = format!("{}/{}", installdir, &pkg.0);

                                                // Setup copy options for copying the extracted directory
                                                let mut options = CopyOptions::new();
                                                options.overwrite = true;
                                                options.copy_inside = true;

                                                // Copy the extracted package from extractdir to pkgidir
                                                match copy(&extractdir, &pkgidir, &options) {
                                                    Ok(_) => {
                                                        println!(
                                                            "{}: {}",
                                                            "Package installed successfully"
                                                                .green()
                                                                .bold(),
                                                            &pkg.0
                                                        );

                                                        // Remove the temporary package file
                                                        match fs::remove_file(&tmpfp) {
                                                            Ok(_) => {},
                                                            Err(e) => eprintln!(
                                                                "{}: {}",
                                                                "Failed to remove temporary package file".red().bold(),
                                                                e
                                                            ),
                                                        }

                                                        // Create symlinks from the package installation directory
                                                        let symlinks =
                                                            create_symlinks(&pkgidir, pkg.1);
                                                        // Set executable permissions for each created symlink
                                                        for ln in symlinks {
                                                            match fs::set_permissions(&ln, fs::Permissions::from_mode(0o755)) {
                                                                Ok(_) => {},
                                                                Err(e) => eprintln!(
                                                                    "{}: {}",
                                                                    "Failed to set executable permissions on symlink".red().bold(),
                                                                    e
                                                                ),
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "{}: {}",
                                                            "Failed to copy extracted package".red().bold(),
                                                            e
                                                        );
                                                        exit(1);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "{}: {}",
                                                    "Failed to extract package".red().bold(),
                                                    e
                                                );
                                                exit(1);
                                            }
                                        }
                                    } else {
                                        let pkgdir = format!("{}/{}", installdir, &pkg.1.name);

                                        match fs::rename(&tmpfp, &pkgdir) {
                                            Ok(_) => {
                                                match fs::set_permissions(&pkgdir, fs::Permissions::from_mode(0o755)) {
                                                    Ok(_) => println!("{}{}","Successfully installed package: ".green().bold(), &pkg.0),
                                                    Err(e) => {
                                                        eprintln!(
                                                            "{}: {}",
                                                            "Failed to set executable permissions on binary".red().bold(),
                                                            e
                                                        );
                                                        exit(1);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "{}: {}",
                                                    "Failed to rename package".red().bold(),
                                                    e
                                                );
                                                exit(1);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "{}: {}",
                                        "Failed to download package".red().bold(),
                                        e
                                    );
                                    exit(1);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Failed to read package index".red().bold(), e);
                    exit(1);
                }
            }
        }
        _ => {
            eprintln!("{}: {}\n{}", "Invalid command".red().bold(), cmd,"run \"gob help\" for more information");
            exit(1);
        }
    }
}

pub fn getpkgindex() -> String {
    let host = match sys_info::hostname() {
        Ok(host) => host,
        Err(_) => {
            eprintln!("{}", "Error getting hostname".red().bold());
            exit(1);
        }
    };
    let indexfl = format!("/home/{}/.gobbled.gob", host);
    match File::open(indexfl) {
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
