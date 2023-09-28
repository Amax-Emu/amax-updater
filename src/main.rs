use anyhow::{anyhow, Result};
use async_compat::Compat;
use console::style;
use console::Style;
use console::{StyledObject, Term};
use env_logger::{Builder, Target};
use log::{info, warn, LevelFilter};
use native_dialog::FileDialog;
use native_dialog::MessageDialog;
use native_dialog::MessageType;
use std::process::Command;
use std::io::Write;
use std::{
    fs, 
    path::{Path, PathBuf},
};

fn main() {
    fn level_styler(level: log::Level) -> StyledObject<String> {
        match level {
            log::Level::Error => style("E".to_owned()).red(),
            log::Level::Warn => style("W".to_owned()).yellow(),
            log::Level::Info => style("I".to_owned()).white(),
            log::Level::Debug => style("D".to_owned()).white(),
            log::Level::Trace => style("T".to_owned()).white(),
        }
    }

    let amax_logo = "
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⣤⣤⡤⠄⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⡿⠋⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⠟⠋⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠈⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⢹⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⢿⣿⣿⣿⣿⣿⣧⠀⠀⢀⣠⣾⣿⣿⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣷⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⣠⣿⣿⣿⣿⣿⣿⣿⣿⡀⠀⠀⠀⠈⢿⣿⣿⣿⣿⣿⣆⣴⣿⣿⡿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣿⣿⠟⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⡀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⢀⣴⣿⡿⢿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠈⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⣿⣿⠏⠀⢸⣿⣿⣿⣿⣿⣿⣇⠀⠀⢰⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣼⣿⣿⢳⣿⣿⣿⣿⣿⡇⠀⢠⣾⣿⡟⠁⢸⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣼⣿⡿⠃⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⠀⠀⣾⣿⡿⠸⣿⣿⣿⣿⣿⣷⠀⣼⣿⡿⠁⣾⣿⣿⣿⣿⣿⠀⣰⣿⣿⠏⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⡆⠀⠀⢀⣤⣾⣿⣿⣿⣿⣿⣿⣿⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⡿⠿⠿⠿⠿⠿⢿⣿⣿⣿⣿⣿⣿⡇⢸⣿⣿⠇⠀⢿⣿⣿⣿⣿⣿⣾⣿⡿⠁⢰⣿⣿⣿⣿⣿⡇⣴⣿⡿⠿⠿⠿⠿⠿⢿⣿⣿⣿⣿⣿⣿⣧⢀⣴⣿⣿⡿⠋⠹⣿⣿⣿⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣰⣿⣿⠏⠀⠀⠀⠀⠀⠀⠸⣿⣿⣿⣿⣿⣿⣷⣿⣿⡿⠀⠀⢸⣿⣿⣿⣿⣿⣿⡿⠁⠀⣾⣿⣿⣿⣿⣿⣾⣿⡟⠁⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⢹⣿⣿⣿⣿⣿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢀⣼⣿⡿⠃⠀⠀⠀⠀⠀⠀⠀⢀⣶⡖⠒⠂⣰⣶⡀⠀⣴⣶⠆⣴⣶⠀⣶⡆⠀⠀⢰⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⣾⣿⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⠓⠒⢀⡟⣿⣇⡼⣿⡿⢠⣿⠇⢸⣿⠃⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠐⠛⠛⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⠛⠒⠂⠘⠁⠙⠛⠁⠛⠃⠈⠛⠒⠚⠉⠀⠀⠀⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
";
    let cyan = Style::new().cyan().bold();
    let error = Style::new().red();

    let mut builder = Builder::from_default_env();
    builder.format(|buf, record| {
        writeln!(buf, "{} - {}", level_styler(record.level()), record.args())
    });
    builder.filter_level(LevelFilter::Info);
    builder.target(Target::Stdout);
    builder.init();

    let term = Term::stdout();

    term.write_line(amax_logo)
        .expect("Failed to print into term!");
    println!("{}", cyan.apply_to("Amax Emu updater v0.1"));
    println!("");

    let blur_path = match get_blur_path() {
        Ok(path) => path,
        Err(e) => {
            println!("{} ({})", error.apply_to("Failure!"), e);
            return;
        }
    };

    println!("{}", cyan.apply_to("Checking if update is needed..."));
    let mut updater = amax_updater_client::AmaxUpdateClient::new(blur_path);
    let update_is_required = match updater.perform_update() {
        Ok(update_is_required) => update_is_required,
        Err(e) => {
            println!(
                "{} ({}) {}",
                error.apply_to("Failure!"),
                e,
                "Probably something wrong with internet connection."
            );
            return;
        }
    };

    if !update_is_required {
        println!("{}", cyan.apply_to("You have the latest version! Exiting"));
        return;
    }

    println!(
        "{}",
        cyan.apply_to("Update is avalaible! Updating files...")
    );

    let update_zip_path = updater.temp_path.join("amax_client_files.zip");

    smol::block_on(Compat::new(async {
        updater
            .download_file(
                "https://cs.amax-emu.com/amax_client_files.zip",
                update_zip_path.to_str().unwrap(),
            )
            .await
            .expect("Failed to download update files!");
    }));

    updater.create_backup();
    updater
        .unpack_update(update_zip_path)
        .expect("Failed to unpack update files!");
    updater.apply_update();

    if cfg!(windows) {
      Command::new("taskkill")
      .args(&["/IM", "Blur.exe", "/F"])
      .spawn()
      .expect("Failed to execute taskkill command for Blur");
    }


    let _yes = MessageDialog::new()
    .set_type(MessageType::Info)
    .set_title("Amax Emu updater")
    .set_text("Update was complete!")
    .show_alert()
    .unwrap();


}

fn get_blur_path() -> Result<PathBuf> {
    match fs::metadata(Path::new("./UpdateDirectory.txt")) {
        Ok(_) => {
            info!("Found UpdateDirectory.txt. Running in Blur updater mode");
            return Ok(PathBuf::from(
                String::from_utf8(fs::read(Path::new("./UpdateDirectory.txt")).unwrap())
                    .expect("Failed to process path in UpdateDirectory.txt"),
            ));
        }
        Err(_) => {}
    }

    match fs::metadata(Path::new("./Blur.exe")) {
        Ok(_) => {
            info!("Found Blur.exe!");
            return Ok(PathBuf::from("."));
        }
        Err(_) => {}
    }

    warn!("Failed to find Blur installation folder. Please select it manually");

    let path_temp = FileDialog::new().show_open_single_dir().unwrap();

    match path_temp {
        Some(path) => return Ok(path),
        None => {}
    };

    Err(anyhow!("Failed to find Blur installation folder. Exiting!"))
}
