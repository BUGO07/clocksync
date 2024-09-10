mod utils;
use utils::*;

use std::env;
use std::process;
use std::process::Command;

const NTP_SERVER: &str = "time.windows.com:123";

#[tokio::main]
async fn main() {
    let mut server = NTP_SERVER.to_string();
    let mut timezone_offset = (0, 0);

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args.len() < 3 || args[1] != "--server" {
            eprintln!(
                "Usage: {} --server <time server> --zone <timezone>",
                args[0]
            );
            process::exit(1);
        }
        server = args[2].clone();

        if args.len() > 3 && args[3] == "--zone" {
            timezone_offset = match parse_timezone(&args[4]) {
                Ok(offset) => offset,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };
        }
    }

    let ntp_time = match fetch_time_from_server(&server).await {
        Ok(time) => time,
        Err(_) => match fetch_time_from_server(NTP_SERVER).await {
            Ok(time) => time,
            Err(_) => {
                eprintln!("Failed to get time from default server.");
                process::exit(1);
            }
        },
    };

    let total_seconds =
        ntp_time + (timezone_offset.0 as u64 * 3600) + (timezone_offset.1 as u64 * 60);

    let (year, month, day, hours, minutes, seconds) = convert_to_datetime(total_seconds);

    if let Err(e) = set_system_time(year, month, day, hours, minutes, seconds) {
        eprintln!("Failed to set system time: {}", e);
        process::exit(1);
    } else {
        println!(
            "Synchronized date and time (UTC{:+}): {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            timezone_offset.0, year, month, day, hours, minutes, seconds
        );
    }
}

fn set_system_time(
    year: u16,
    month: u8,
    day: u8,
    hours: u8,
    minutes: u8,
    seconds: u8,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let date_str = format!("{:02}-{:02}-{:04}", month, day, year);

        let date = Command::new("cmd")
            .args(&["/C", "date", &date_str])
            .status()
            .map_err(|e| e.to_string())?;

        if !date.success() {
            return Err("Failed to set date".to_string());
        }

        let time = Command::new("cmd")
            .args(&["/C", "time", &time_str])
            .status()
            .map_err(|e| e.to_string())?;

        if !time.success() {
            return Err("Failed to set time".to_string());
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let datetime_str = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, seconds
        );
        let status = Command::new("date")
            .args(&["-s", &datetime_str])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to set date and time".to_string());
        }
    }

    Ok(())
}
