use std::io::{stdin, stdout, Write};
use colored::Colorize;

pub fn print_update_bar(chunks_count: f64, total: f64) {
    let mut hashtags = String::new();
    let mut blankspaces = String::new();
    let percentage = (chunks_count / total) * 100.0;
    for _ in 0..percentage.round() as u64 {
        hashtags += "#";
    }
    for _ in (percentage.round() as u64)..100 {
        blankspaces += "_";
    }
    println!("{}c", 27 as char); // Clears the screen
    println!("{}", "1. Downloading latest version".underline());
    println!("{} [{}{}] ({:.2}%)", "Downloading".cyan(), hashtags.green(), blankspaces.dimmed(), percentage);
}

pub fn print_help(updater_version: &str) {
    println!("{} v.{}", "Playdate SDK Updater for Linux".underline(), updater_version.green());
    println!("Install and Update the Playdate SDK using an interactive CLI.");
    println!("{}", "---- USAGE ----".bold().underline());
    println!("playdate-sdk-updater : start an interactive CLI to guide you through the SDK setup.");
    println!("--clean : ignore cached files");
    println!("--install-dir=[directory] : install in specified directory (only for 1st install)");
    println!("--help : show this help page");
    println!("{}", "---- LEGAL ----".bold().underline());
    println!("Copyright (c) Oxey405 - 2025 Under MIT License");
    println!("Not affiliated with Panic Inc. Playdate is a trademark of Panic Inc.");
    println!("THIS PROGRAM IS DISTRIBUTED \"AS IS\" AND COMES WITHOUT ANY WARRANTY TO THE EXTENT PERMITTED BY THE LAW");
    println!("For any question regarding this software, please send an e-mail at hello@oxey405.com")
}

pub fn yes_no_prompt(prompt: &String, default: bool) -> bool {
    print!("{} > ", prompt.trim());
    let _ = stdout().flush();
    let mut s = String::new();
    match stdin().read_line(&mut s) {
        Ok(_) => {
            println!("{}", s);
            if s.trim().to_lowercase() == "y" {
                return true;
            } else if s.trim().to_lowercase() == "n" {
                return false;
            } else {
                return default;
            }
        }

        Err(_) => return default,
    }
}