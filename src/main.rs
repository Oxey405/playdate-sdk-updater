use std::{
    collections::HashMap,
    env,
    fs::{self, create_dir, exists, read_dir, remove_dir_all, File},
    io::{stdin, stdout, ErrorKind, Write},
    path::Path,
    process::exit,
};

use flate2::read::GzDecoder;
use tar::Archive;
use tokio::fs::try_exists;
use colored::Colorize;


mod utils;
use utils::{print_update_bar, print_help, yes_no_prompt};

const DOWNLOAD_FILE_PATH: &str = "/tmp/playdatesdk.tar.gz";
const UPDATER_VERSION: &str = "1.0.0";

#[tokio::main]
async fn main() {
    println!("{} v.{}", "PLAYDATE SDK UPDATER".yellow().bold().underline(), format!("{}", UPDATER_VERSION).green());
    println!("(Made by {}, {} with Panic Inc.)", "Oxey405".underline().blink(), "NOT affiliated".bold());
    if has_flag("--help") {
        print_help(UPDATER_VERSION);
        exit(0);
    }


    let sdk_path = get_sdk_path();

    let update_info = get_update_info().await;
    if !update_info.contains_key("buttonURL") {
        println!("Critical Error : Could not find downlaod URL.");
        return;
    }

    if sdk_path.is_err() || !Path::new(&sdk_path.unwrap()).exists() {
        println!("There seems to be no previous install of the Playdate SDK on this computer");
        if yes_no_prompt(&"Install latest version ? (Y/n)".to_string(), true) {
            println!("Please read the terms and conditions for the SDK (see https://play.date/dev)");
            if !yes_no_prompt(&"Have you read and accepted the Playdate SDK's terms ? (y/N)".to_string(), false) {
                println!("Please accept the terms and conditions for the SDK");
                exit(0);
            }

            let custom_install_dir = get_param("--install-dir");
            if custom_install_dir != "" {
                println!("Installing in directory {}", custom_install_dir.underline());
                handle_install(custom_install_dir, update_info.get("buttonURL").unwrap().to_string()).await;
                exit(0);
            }

            print!("Enter desired install path ({}) > ", "absolute path".bold());
            let _ = stdout().flush();
            let mut path_to_set = String::new();
            match stdin().read_line(&mut path_to_set) {
                Ok(_) => {
                    handle_install(path_to_set, update_info.get("buttonURL").unwrap().to_string()).await;
                    exit(0);
                }

                Err(_) => {
                    println!("Invalid input, exiting...");
                    exit(-1);
                }
            }
        } else {
            println!("{}", "The SDK will NOT be updated. Goodbye.".red());
            exit(0);
        }
    }

    let mut uncertain_update = true;
    if !update_info.contains_key("ID") {
        println!(
            "Could not check if newer version is available... do you want to download anyway ? (y/N)"
        );
    } else if update_info.get("ID").unwrap() != "" {
        println!(
            "Newer version is available ({} --> {}) ! Do you want to install it ? (Y/n)",
            get_sdk_version().unwrap().magenta(),
            update_info.get("ID").unwrap().bright_green()
        );
        uncertain_update = false;
    } else {
        println!(
            "You are already up-to-date (version {})!",
            get_sdk_version().unwrap_or("-.-.-".to_string()).green().bold()
        );
        return;
    }

    let _ = stdout().flush();
    if yes_no_prompt(&" > ".to_string(), !uncertain_update) {
        update_sdk(
            update_info.get("buttonURL").unwrap().to_string(),
            get_sdk_path().expect("No SDK path available (PLAYDATE_SDK_PATH variable not set)"),
        )
        .await;
    } else {
        println!("{}", "The SDK will NOT be updated. Goodbye.".red());
        return;
    }
}

fn get_sdk_path() -> Result<String, env::VarError> {
    env::var("PLAYDATE_SDK_PATH")
}

fn get_sdk_version() -> Result<String, std::io::Error> {
    let sdk_path = get_sdk_path().unwrap();
    let version = fs::read_to_string(sdk_path + "/VERSION.txt");
    if version.is_err() {
        return Err(version.err().unwrap());
    }
    Ok(version.unwrap().trim().to_string())
}

async fn get_update_info() -> HashMap<String, String> {
    let version = get_sdk_version().unwrap_or("".to_string());
    println!("Found Playdate SDK version {}", &version.bold().yellow());
    let url = String::from(
        "https://panic.com/updates/soapbox.php?app=Playdate%20Simulator&platform=linux&appver=",
    ) + &version;
    println!("Checking for updates from {}", &url.dimmed());
    let resp = reqwest::get(url)
        .await
        .expect("Could not check updates from server")
        .json::<HashMap<String, String>>()
        .await
        .expect("Could not parse update server's response");
    return resp;
}

async fn download_sdk_from_url(url: &String) {
    let mut file = File::create(DOWNLOAD_FILE_PATH)
        .expect("Critical Error : Could not create temporary file.");
    let mut resp = reqwest::get(url)
        .await
        .expect("Could not download SDK from URL");
    let mut size = resp.content_length().unwrap_or_default();
    if size == 0 {
        size = 1
    }
    let mut chunks_count = 0;
    while let Some(chunk) = resp
        .chunk()
        .await
        .expect("Something went wrong when downloading the file.")
    {
        print_update_bar(chunks_count as f64, size as f64);
        let _ = file.write_all(&chunk);
        chunks_count += chunk.len();
    }

    let _ = file.flush();
    println!("{}", "File downloaded successfully.".green());
}

async fn copy_files(path: &String) -> Result<(), std::io::Error> {
    // First we make sure to backup the last install the user had in case of critical error
    if Path::new(&path).try_exists().unwrap_or(false) {
        println!("{}", format!("Backing up {} to {}", &path, path.clone() + "_backup").dimmed());
        fs::remove_dir_all(path.clone() + "_backup").unwrap_or(());
        let backup_ok = fs::rename(&path, path.clone() + "_backup");
        if backup_ok.is_err() {
            if !yes_no_prompt(&"Could not back up the previous install, continue anyway ? (If this is the first install, type \"y\") (y/N)".to_string(), false) {
            println!("Aborting installation.");
            return Err(std::io::Error::new(ErrorKind::PermissionDenied, "Back up failed and installation was rejected"));
            }
        }
    }

    let dir_result = create_dir(&path);
    if dir_result.is_err() {
        println!("Could not create directory, {}", "aborting installation".red().bold());
        return dir_result;
    }

    let tar_gz = File::open(DOWNLOAD_FILE_PATH)
        .expect("Critical Error : Could not open downloaded version.");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let unpacking = archive.unpack(path.clone() + "_tmp");

    if unpacking.is_err() {
        println!("Could not unpack downloaded version, {}", "aborting installation".red().bold());
        return unpacking;
    }

    println!("{}", "Unpacking done, extracting from unpacked folder".green());


    let contents =
        read_dir(path.clone() + "_tmp").expect("Critical Error : Could not read extracted folder");
    for _file in contents {
        let file = _file.unwrap();
        if file.file_type().unwrap().is_dir() {
            fs::rename(file.path(), &path).expect(
                "Critical Error : Could not extract files from unpacked folder to destination.",
            );
        }
    }

    Ok(())
}

async fn update_sdk(sdk_url: String, sdk_path: String) {
    let path = sdk_path.trim().to_string();
    println!("Updating your Playdate SDK install !");
    println!("{}", "1. Downloading latest version".underline());
    if !try_exists(DOWNLOAD_FILE_PATH).await.unwrap_or(false) || has_flag("--clean"){
        download_sdk_from_url(&sdk_url).await;
    } else {
        println!("latest version was already downloaded (if you want to force re-download, use argument --clean")
    }
    println!("{}", "2. Extracting files into folder".underline());

    let result = copy_files(&path).await;
    if result.is_err() {
        print!("{}", result.err().unwrap());
        println!(
            "An error happened when installing. You can restore your previous installation by renaming the backup folder to the original folder name."
        );
        exit(-1);
    } else {
        println!("{}", "Unpacking successful !".green())
    }

    println!("{}", format!("3. Run the shell script {}/setup.sh to finish setup", &path).underline());


    if yes_no_prompt(&"Run the shell command (requires being root) ? (Y/n)".to_string(), true) {
        let mut script = std::process::Command::new(format!("{}/setup.sh", &path));
        let r = script.spawn();
        if r.is_err() {
            println!("The script crashed ; you may have to run it yourself with root privileges.");

        }
    } else {
        println!("You'll have to {} to finish your install", "run the shell script yourself".bold().on_red().white());
        println!("Typically, you would run the following \n {}", format!("sudo {}", format!("{}/setup.sh", &path)).bright_green().on_black())
    }
    println!(
        "4. Don't forget to set the environment variable ! {}{}",
        "PLAYDATE_SDK_PATH=".blue().bold().underline(),
        &path.blue().bold()
    );
    cleanup(path);

    
}

async fn handle_install(path: String, sdk_url: String) {
    let sdk_path_env: &Path = Path::new(&path);
    assert!(
        sdk_path_env.extension().is_none(),
        "Provided path is not a directory. Aborting install"
    );
    assert!(
        !sdk_path_env.exists(),
        "Provided path already exists. Aborting install"
    );
    update_sdk(
        sdk_url,
        sdk_path_env.to_str().unwrap().trim().to_string(),
    )
    .await;
    exit(0);
}

fn cleanup(path: String) {
    print!("Cleaning up...");
    if exists(path.clone() + "_tmp").unwrap_or(false) {
        remove_dir_all(path.clone() + "_tmp").unwrap();
    }
    println!("Goodbye !")

}

fn has_flag(arg: &str) -> bool {
    env::args().into_iter().find(|a| a == arg).is_some()
}

fn get_param(param: &str) -> String {
    let args = env::args();
    for _arg in args.into_iter() {
        let mut arg = _arg.clone();
        if arg.starts_with(param) {
            return arg.split_off(param.len()+1)
        }
    }
    return String::new();
}