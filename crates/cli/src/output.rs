use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::io::Write;

pub enum OutputFormat {
    Json,
    Table,
}

pub fn print_json<T: Serialize>(data: &T) {
    println!("{}", serde_json::to_string_pretty(data).unwrap());
}

pub fn print_colored(status: &str, text: &str) {
    let colored = match status.to_lowercase().as_str() {
        "success" | "ok" | "verified" | "true" => text.green(),
        "error" | "failed" | "false" => text.red(),
        "warning" | "warn" | "pending" => text.yellow(),
        "info" => text.cyan(),
        _ => text.normal(),
    };
    println!("{}", colored);
}

pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("No data.");
        return;
    }
    let col_count = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }
    for (i, header) in headers.iter().enumerate() {
        print!("{}", header.bold());
        if i < col_count - 1 {
            let pad = widths[i].saturating_sub(header.len()) + 3;
            print!("{}", " ".repeat(pad));
        }
    }
    println!();
    let total_width: usize = widths.iter().map(|w| w + 3).sum::<usize>().saturating_sub(3);
    println!("{}", "-".repeat(total_width));
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            print!("{}", cell);
            if i < col_count - 1 {
                let pad = widths[i].saturating_sub(cell.len()) + 3;
                print!("{}", " ".repeat(pad));
            }
        }
        println!();
    }
}

pub fn print_key_value(key: &str, value: &str) {
    println!("{}: {}", key.cyan().bold(), value);
}

pub fn create_progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(msg.to_string());
    pb
}

pub fn print_success(msg: &str) {
    println!("{} {}", "[OK]".green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "[ERROR]".red().bold(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "[WARN]".yellow().bold(), msg);
}
