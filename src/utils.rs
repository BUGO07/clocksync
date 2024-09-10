use std::str::FromStr;
use std::time::UNIX_EPOCH;

const NTP_EPOCH_OFFSET: u64 = 2208988800;

pub async fn fetch_time_from_server(server: &str) -> Result<u64, String> {
    let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(sock) => sock,
        Err(_) => return Err("Failed to bind UDP socket.".to_string()),
    };

    socket
        .connect(server)
        .await
        .map_err(|_| "Failed to connect to server.".to_string())?;
    let request = [0x1b; 48]; // NTP request packet
    socket
        .send(&request)
        .await
        .map_err(|_| "Failed to send request.".to_string())?;

    let mut response = [0; 48];
    socket
        .recv(&mut response)
        .await
        .map_err(|_| "Failed to receive response.".to_string())?;

    // Extract NTP timestamp (seconds since 1900-01-01)
    let timestamp = u32::from_be_bytes(response[40..44].try_into().unwrap()) as u64;
    let unix_timestamp = timestamp - NTP_EPOCH_OFFSET;

    Ok(unix_timestamp)
}

// Parse the timezone argument
pub fn parse_timezone(zone: &str) -> Result<(i64, i64), &'static str> {
    let prefix = "UTC";
    if !zone.starts_with(prefix) {
        return Err("Timezone must start with 'UTC'");
    }

    let zone = &zone[prefix.len()..];
    let (sign, rest) = zone.split_at(1);
    let sign = match sign {
        "+" => 1,
        "-" => -1,
        _ => return Err("Timezone must be in the format UTC±H:M"),
    };

    let mut parts = rest.split(':');
    let hours = match parts.next().and_then(|h| i64::from_str(h).ok()) {
        Some(h) => h * sign,
        None => return Err("Invalid hours in timezone"),
    };

    let minutes = match parts.next().and_then(|m| i64::from_str(m).ok()) {
        Some(m) => m,
        None => return Err("Invalid minutes in timezone"),
    };

    Ok((hours, minutes))
}

// Convert seconds to date and time components
pub fn convert_to_datetime(total_seconds: u64) -> (u16, u8, u8, u8, u8, u8) {
    // Convert total seconds to SystemTime
    let datetime = UNIX_EPOCH + std::time::Duration::from_secs(total_seconds);
    let datetime = datetime.duration_since(UNIX_EPOCH).unwrap();

    let total_seconds = datetime.as_secs();
    let days_since_epoch = total_seconds / 86400;
    let seconds_today = total_seconds % 86400;

    let hours = (seconds_today / 3600) as u8;
    let minutes = ((seconds_today % 3600) / 60) as u8;
    let seconds = (seconds_today % 60) as u8;

    let mut days = days_since_epoch;
    let mut year = 1970;
    while days >= 365 {
        let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            1
        } else {
            0
        };
        let days_in_year = 365 + leap;
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let mut month = 1;
    while days >= days_in_month(month, year) {
        days -= days_in_month(month, year);
        month += 1;
    }

    let day = (days + 1) as u8;
    (year as u16, month, day, hours, minutes, seconds)
}

// Helper function to determine days in month
pub fn days_in_month(month: u8, year: u64) -> u64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}
