use std::{fs::File, io, path::Path, error::Error};

pub fn extract_package(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(pkg_path);
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    
    if filename.ends_with(".zip") {
        extract_zip(pkg_path, output_dir)
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        extract_tar_gz(pkg_path, output_dir)
    } else if filename.ends_with(".tar.xz") {
        extract_tar_xz(pkg_path, output_dir)
    } else if filename.ends_with(".tar.bz2") {
        extract_tar_bz2(pkg_path, output_dir)
    } else if filename.ends_with(".tar") {
        extract_tar(pkg_path, output_dir)
    } else {
        Err("Unsupported archive format".into())
    }
}

fn extract_zip(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    use zip::ZipArchive;
    let file = File::open(pkg_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(output_dir).join(file.mangled_name());
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    let file = File::open(pkg_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(output_dir)?;
    Ok(())
}

fn extract_tar_xz(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    use xz2::read::XzDecoder;
    use tar::Archive;
    let file = File::open(pkg_path)?;
    let decompressor = XzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(output_dir)?;
    Ok(())
}

fn extract_tar_bz2(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    use bzip2::read::BzDecoder;
    use tar::Archive;
    let file = File::open(pkg_path)?;
    let decompressor = BzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(output_dir)?;
    Ok(())
}

fn extract_tar(pkg_path: &str, output_dir: &str) -> Result<(), Box<dyn Error>> {
    use tar::Archive;
    let file = File::open(pkg_path)?;
    let mut archive = Archive::new(file);
    archive.unpack(output_dir)?;
    Ok(())
}
