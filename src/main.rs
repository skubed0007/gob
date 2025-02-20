use colored::Colorize;
use extract::extract_package;
use fs_extra::dir::{copy, CopyOptions};
use getpkg::getpkg;
use mk_symlink::create_symlinks;
use search::searchpkg;
use std::{
    env::args, fs::{self, File}, io::{read_to_string, Read}, os::unix::fs::PermissionsExt, path::Path, process::exit
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
    let host = match sys_info::hostname() {
        Ok(host) => host,
        Err(_) => {
            eprintln!("{}", "Error getting hostname".red().bold());
            exit(1);
        }
    };
    let indexfl = format!("/home/{}/.gobbled.gob", host);
    let installdir = format!("/home/{}/.gob", host);
    if !Path::new(&installdir).exists() {
        match fs::create_dir(&installdir){
            Ok(_) => println!("{}: {}", "Created install directory".green().bold(), installdir),
            Err(e) => {
                eprintln!("{}: {}", "Failed to create install directory".red().bold(), e);
                exit(1);
            }
        }
    }

    match cmd.as_str() {
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
            match fs::read_to_string(indexfl){

                Ok(pkgindex_str) => {
                    let mut pkgindex = parse_pkg_index::ppkgi(&pkgindex_str);
                    searchpkg(&search_terms, &pkgindex_str);
                    for pkg in &mut pkgindex{
                        if search_terms.iter().any(|term| &pkg.0 == &term){
                            //check if package is extractable and thus create proper install dir and links
                            let binfolder_file = {
                                if pkg.1.extractable{
                                    format!("{}/{}", installdir, &pkg.0)
                                }
                                else {
                                    format!("{}/{}",installdir,&pkg.1.name)
                                }
                            };
                            match getpkg(&pkg.1).await{
                                Ok(tmpfp) => {
                                    if pkg.1.extractable{
                                        let extractdir = format!("/tmp/{}", &pkg.1.name);
                                        match extract_package(&tmpfp,&extractdir){
                                            Ok(_) => {
                                                let pkgidir = format!("{}/{}", installdir, &pkg.0);
                                                let mut options = CopyOptions::new();
                                                options.overwrite = true;
                                                options.copy_inside = true;
                                                match copy(&extractdir, &pkgidir, &options){
                                                    Ok(_) => {
                                                        println!("{}: {}", "Package extracted successfully".green().bold(), &pkg.0);
                                                        let symlinks =  create_symlinks(&pkgidir,pkg.1);
                                                        for ln in symlinks {
                                                            if let Err(e) = fs::set_permissions(&ln, fs::Permissions::from_mode(0o755)) {
                                                                eprintln!("{}: {}", "Failed to set executable permissions on symlink".red().bold(), e);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("{}: {}", "Failed to copy extracted package".red().bold(), e);
                                                        exit(1);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("{}: {}", "Failed to extract package".red().bold(), e);
                                                exit(1);
                                            }
                                        }
                                    }
                                    else {
                                        let pkgdir = format!("{}/{}", installdir, &pkg.1.name);
                                        match fs::write(&pkgdir, &tmpfp){
                                            Ok(_) => {
                                                println!("{}: {}", "Package downloaded successfully".green().bold(), &pkg.0);
                                                let symlinks = create_symlinks(&pkgdir, pkg.1);
                                                for ln in symlinks {
                                                    if let Err(e) = fs::set_permissions(&ln, fs::Permissions::from_mode(0o755)) {
                                                        eprintln!("{}: {}", "Failed to set executable permissions on symlink".red().bold(), e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("{}: {}", "Failed to write package".red().bold(), e);
                                                exit(1);
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{}: {}", "Failed to download package".red().bold(), e);
                                    exit(1);
                                },
                            }
                        }
                    }
                }
                Err(e ) => {
                    eprintln!("{}: {}", "Failed to read package index".red().bold(), e);
                    exit(1);
                }
            }
        }
        _ => {}
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
