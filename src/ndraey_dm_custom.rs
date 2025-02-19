use colored::Colorize;
use futures_util::StreamExt;
/// A simple module for downloading large files
/// by NDRAEY (c) 2022
// use reqwest;
use std::fs::File;
use std::io::Write;
use std::io::stdout;
use std::time::{SystemTime, UNIX_EPOCH};

/// Downloads a file from the given URL in chunks and displays a progress bar.
///
/// This function downloads a file from the specified URL and saves it to the given path.
/// The download progress is displayed in the terminal with a progress bar, showing the
/// percentage completed, download speed, and the amount of data downloaded.
///
/// # Arguments
///
/// * `url` - A `String` representing the URL of the file to be downloaded.
/// * `path` - A `String` representing the file path where the downloaded file will be saved.
///
/// # Returns
///
/// * `bool` - Returns `true` if the download was successful, otherwise returns `false`.
///
/// # Errors
///
/// This function will return `false` and print an error message if:
/// - The GET request to the URL fails.
/// - The file cannot be created at the specified path.
/// - There is an error while downloading a chunk of the file.
/// - There is an error while writing a chunk to the file.
///
/// # Example
///
/// ```rust
/// use tokio;
///
/// #[tokio::main]
/// async fn main() {
///     let url = "https://example.com/largefile.zip".to_string();
///     let path = "/path/to/save/largefile.zip".to_string();
///
///     let success = progress(url, path).await;
///     if success {
///         println!("Download completed successfully!");
///     } else {
///         println!("Download failed.");
///     }
/// }
/// ```
///
/// # Dependencies
///
/// This function requires the following dependencies:
/// - `reqwest` for making HTTP requests.
/// - `futures_util` for handling asynchronous streams.
/// - `colored` for colored terminal output.
///
/// # Notes
///
/// - The progress bar updates every second to show the current download speed.
/// - The download speed is displayed in kilobytes per second (kB/s).
/// - The progress bar consists of 20 characters, with the completed portion represented by `─`
///   and the remaining portion represented by spaces.
pub async fn progress(url: String, path: String) -> bool {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .unwrap();

    let res = match client.get(url.clone()).send().await {
        Ok(response) => response,
        Err(err) => {
            println!(
                "[ndraey_downloader] Failed to send request! (Error: {})",
                err
            );
            return false;
        }
    };

    let total_size = match res.content_length() {
        Some(size) => size,
        None => {
            println!("[ndraey_downloader] Failed to get content length!");
            return false;
        }
    };

    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(err) => {
            println!("[ndraey_downloader] Failed to create file '{}': {}", path, err);
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
                println!("[ndraey_downloader] Error while downloading file: {}", err);
                return false;
            }
        };

        if let Err(err) = file.write_all(&chunk) {
            println!("[ndraey_downloader] Error while writing to file '{}': {}", path, err);
            return false;
        }

        downloaded += chunk.len() as u64;
        downloaded_in_sec += chunk.len() as usize;

        let percent = (downloaded as f64 / total_size as f64) * 100_f64;

        let newtime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time error")
            .as_secs();

        if newtime > sys_time {
            speed = downloaded_in_sec;
            downloaded_in_sec = 0;
            sys_time = newtime;
        }

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
