use colored::Colorize;
use futures_util::StreamExt;
use reqwest;
use std::fs::File;
use std::io::{stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};

fn format_bytes(bytes: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes < KB {
        format!("{:.2} B", bytes)
    } else if bytes < MB {
        format!("{:.2} kB", bytes / KB)
    } else if bytes < GB {
        format!("{:.2} MB", bytes / MB)
    } else {
        format!("{:.2} GB", bytes / GB)
    }
}

pub async fn progress(url: String, path: String) -> bool {
    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            println!("[DOWNLOADER] Failed to build client! (Error: {})", err);
            return false;
        }
    };

    let res = match client.get(url.clone()).send().await {
        Ok(response) => response,
        Err(err) => {
            println!("[DOWNLOADER] Failed to send request! (Error: {})", err);
            return false;
        }
    };

    let total_size = match res.content_length() {
        Some(size) => size,
        None => {
            println!("[DOWNLOADER] Failed to get content length!");
            return false;
        }
    };

    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(err) => {
            println!("[DOWNLOADER] Failed to create file '{}': {}", path, err);
            return false;
        }
    };

    let mut downloaded: u64 = 0;
    let mut downloaded_in_sec: usize = 0;
    let mut stream = res.bytes_stream();

    let mut sys_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time error")
        .as_secs();
    let mut speed: usize = 0;

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(chunk) => chunk,
            Err(err) => {
                println!("[DOWNLOADER] Error while downloading file: {}", err);
                return false;
            }
        };

        if let Err(err) = file.write_all(&chunk) {
            println!(
                "[DOWNLOADER] Error while writing to file '{}': {}",
                path, err
            );
            return false;
        }

        downloaded += chunk.len() as u64;
        downloaded_in_sec += chunk.len() as usize;

        let percent = (downloaded as f64 / total_size as f64) * 100.0;

        let new_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time error")
            .as_secs();

        if new_time > sys_time {
            speed = downloaded_in_sec;
            downloaded_in_sec = 0;
            sys_time = new_time;
        }

        print!(
            "\r[{}/s] [{} / {} ~ ({}%)]\x1b[K",
            format_bytes(speed as f64).yellow().bold(),
            format_bytes(downloaded as f64).green(),
            format_bytes(total_size as f64).green().bold(),
            (percent as usize).to_string().bold().red()
        );
        stdout().flush().unwrap();
    }
    println!();
    true
}
