use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{info, warn};
use std::cmp::min;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{self, Write as io_write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::tempdir;
use version_compare::{compare_to, Cmp};
pub struct AmaxUpdateClient {
    http_client: reqwest::blocking::Client,
    pub temp_path: PathBuf,
    pub blur_path: PathBuf,
    update_files: Vec<PathBuf>,
    remote_version: String,
}

impl AmaxUpdateClient {
    pub fn new(blur_path: PathBuf) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .https_only(true)
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(5))
            .gzip(true)
            .user_agent("Amax Updater Client v0.1")
            .build()
            .unwrap();

        let temp_path = tempdir().unwrap().into_path();

        fs::remove_dir_all(&temp_path).unwrap_or_default();
        fs::create_dir_all(&temp_path).expect("Failed to create temp dir");

        let mut update_files = vec![];
        let mut remote_version = String::from("0.0.0.0.");
        Self {
            http_client,
            temp_path,
            blur_path,
            update_files,
            remote_version,
        }
    }

    pub fn perform_update(&mut self) -> Result<bool> {
        let version_remote = match self.get_remote_version() {
            Ok(version_remote) => version_remote,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to get remote version - {}. Aborting update",
                    e.to_string()
                ))
            }
        };

        let version_local = match self.get_local_version() {
            Ok(version_local) => version_local,
            Err(e) => {
                warn!("Failed to get version of local files.");
                String::from("None")
            }
        };
        info!(
            "Remote version - {} | Local version - {}",
            version_remote, version_local
        );

        if version_local == "None" {
            return Ok(true);
        }

        let update_needed = compare_to(version_remote, version_local, Cmp::Gt).unwrap_or(false);

        if update_needed {
            return Ok(true);
        } else {
            return Ok(false);
        }

        Ok(false)
    }

    fn get_remote_version(&mut self) -> Result<String> {
        let response = match self
            .http_client
            .get("https://cs.amax-emu.com/version.txt")
            .send()
        {
            Ok(response) => response,
            Err(e) => return Err(anyhow!(e.to_string())),
        };

        match response.text() {
            Ok(version_string) => {
                let temp = version_string.replace("\n", "");
                self.remote_version = temp.to_owned();
                Ok(temp)
            }
            Err(e) => return Err(anyhow!(e.to_string())),
        }
    }

    fn get_local_version(&mut self) -> Result<String> {
        let version_path = &self.blur_path.join("amax").join("version");

        match fs::read_to_string(version_path) {
            Ok(local_version) => Ok(local_version),
            Err(e) => return Err(anyhow!(e.to_string())),
        }
    }

    fn download_update_zip(&mut self, path: &str) -> Result<String> {
        let response = match self
            .http_client
            .get("https://cs.amax-emu.com/amax_client_files.zip")
            .build()
        {
            Ok(response) => response,
            Err(e) => return Err(anyhow!(e.to_string())),
        };

        let mut file = File::create(path)
            .or(Err(format!("Failed to create file '{}'", path)))
            .unwrap();
        let update_content = response.body().unwrap().as_bytes().unwrap();

        file.write_all(update_content)
            .or(Err(format!("Error while writing to file")))
            .unwrap();

        Ok(path.to_owned())
    }

    pub async fn download_file(&mut self, url: &str, path: &str) -> Result<(), String> {
        //todo: move it to main app, this is not the greatest approach.

        let client = reqwest::Client::builder()
            .https_only(true)
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(5))
            .gzip(true)
            .user_agent("Amax Updater Client v0.1")
            .build()
            .unwrap();

        // Reqwest setup
        let res = client
            .get(url)
            .send()
            .await
            .or(Err(format!("Failed to GET from '{}'", &url)))?;
        let total_size = res
            .content_length()
            .ok_or(format!("Failed to get content length from '{}'", &url))?;

        // Indicatif setup
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        // download chunks
        let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.or(Err(format!("Error while downloading file")))?;
            file.write_all(&chunk)
                .or(Err(format!("Error while writing to file")))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Downloaded {} to {}", url, path));
        return Ok(());
    }

    pub fn unpack_update(&mut self, path: PathBuf) -> Result<Vec<PathBuf>> {
        let fname = path;

        let base_path = match fname.parent() {
            Some(path) => path.to_owned(),
            None => PathBuf::new(),
        };

        let mut extracted_files: Vec<PathBuf> = vec![];

        let file = fs::File::open(fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let temp_path = base_path.join(path.to_owned());

                    match path.parent() {
                        Some(parent) => {
                            if parent == Path::new("") {
                                self.update_files.push(path.to_owned())
                            }
                        }
                        None => {}
                    };

                    extracted_files.push(temp_path.to_owned());
                    temp_path
                }
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                println!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        Ok(extracted_files)
    }

    pub fn unpack_zip(path: PathBuf) -> Result<Vec<PathBuf>> {
        let fname = path;

        let base_path = match fname.parent() {
            Some(path) => path.to_owned(),
            None => PathBuf::new(),
        };

        let mut extracted_files: Vec<PathBuf> = vec![];

        let file = fs::File::open(fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let temp_path = base_path.join(path.to_owned());

                    extracted_files.push(temp_path.to_owned());
                    temp_path
                }
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                println!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        Ok(extracted_files)
    }

    pub fn apply_update(&mut self) {
        let _ = fs::remove_dir_all(&self.blur_path.join("amax"));
        for file_path in &self.update_files {
            let _ = fs::rename(
                &self.temp_path.join(file_path),
                &self.blur_path.join(file_path),
            );
        }

        let mut version_file =
            fs::File::create(&self.blur_path.join("amax").join("version")).unwrap();
        io::copy(&mut self.remote_version.as_bytes(), &mut version_file).unwrap();
    }

    pub fn create_backup(&mut self) {
        match fs::metadata(&self.blur_path.join("amax")) {
            Ok(_) => {
                let _ = fs::remove_dir_all(&self.blur_path.join(".amax_bak"));
                let _ = fs::create_dir_all(&self.blur_path.join(".amax_bak"));

                let _ = fs::rename(
                    &self.blur_path.join("amax"),
                    &self.blur_path.join(".amax_bak").join("amax"),
                );
                let _ = fs::rename(
                    &self.blur_path.join("amax").join("version"),
                    &self
                        .blur_path
                        .join(".amax_bak")
                        .join("amax")
                        .join("version"),
                );
                let _ = fs::rename(
                    &self.blur_path.join("d3d9.dll"),
                    &self.blur_path.join(".amax_bak").join("d3d9.dll"),
                );
                let _ = fs::rename(
                    &self.blur_path.join("lua5.1.dll"),
                    &self.blur_path.join(".amax_bak").join("lua5.1.dll"),
                );
                let _ = fs::rename(
                    &self.blur_path.join("discord-rpc.dll"),
                    &self.blur_path.join(".amax_bak").join("discord-rpc.dll"),
                );
            }
            Err(_) => {}
        }
    }

    pub fn move_update_files() {}
}
