use crate::{ndraey_dm_custom, parse_pkg_index::Package};

pub async fn getpkg(pkg: &Package) -> Result<String, Box<dyn std::error::Error>> {
    let pkgname = format!(
        "{}_{}",
        &pkg.name,
        pkg.url.split('/').last().unwrap_or("default_pkg_name").split('?').next().unwrap_or("default_pkg_name")
    );

    let tmp_dir = std::env::temp_dir();
    let tmpfile_path = tmp_dir.join(pkgname);

    match ndraey_dm_custom::progress(pkg.url.clone(), tmpfile_path.display().to_string()).await {
        true => {}
        false => {
            return Err("Failed to download package".into());
        }
    }

    Ok(tmpfile_path.to_string_lossy().to_string())
}
