use colored::Colorize;

use crate::{ndraey_dm_custom, parse_pkg_index::Package};

pub async fn getpkg(pkg: &Package) -> Result<String, Box<dyn std::error::Error>> {
    let pkgname = format!(
        "{}_{}",
        &pkg.name,
        pkg.url.split('/').last().unwrap_or("default_pkg_name").split('?').next().unwrap_or("default_pkg_name")
    );
    let host = match sys_info::hostname() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{}: {}", "Failed to get hostname".red().bold(), e);
            "localhost".to_string()
        }
    };
    let tmpfile_path = format!("/home/{}/.gob/{}", host,pkgname);
    match ndraey_dm_custom::progress(pkg.url.clone(), tmpfile_path.clone()).await {
        true => {}
        false => {
            return Err("Failed to download package".into());
        }
    }

    Ok(tmpfile_path)
}
