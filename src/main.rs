use std::io::BufReader;
use std::io::BufRead;
use std::process::Command;
use std::process::Child;
use std::process::Stdio;

use notify_rust::Notification;
use notify_rust::Urgency;

// nerd font icons
fn discharging_icon_of_percentage(percentage: f32) -> &'static str {
    match ((percentage / 10.0).floor() * 10.0) as i32 {
        0 => "󰂎",
        10 => "󰁺",
        20 => "󰁻",
        30 => "󰁼",
        40 => "󰁽",
        50 => "󰁾",
        60 => "󰁿",
        70 => "󰂀",
        80 => "󰂁",
        90 => "󰂂",
        100 => "󰁹",
        _ => unreachable!()
    }
}

fn charging_icon_of_percentage(percentage: f32) -> &'static str {
    match ((percentage / 10.0).floor() * 10.0) as i32 {
        0 => "󰢟",
        10 => "󰢜",
        20 => "󰂆",
        30 => "󰂇",
        40 => "󰂈",
        50 => "󰢝",
        60 => "󰂉",
        70 => "󰢞",
        80 => "󰂊",
        90 => "󰂋",
        100 => "󰂅",
        _ => unreachable!()
    }
}

struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    fn new() -> Self {
        Self {
            child: Command::new("upower")
            .arg("--monitor")
            .stdout(Stdio::piped())
            .spawn().unwrap(),
        }
    }
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

enum NotificationUrgencyLevel {
    Low,
    Normal,
    Critical
}

fn send_notification(title: &str, body: &str, urgency: Urgency) {
    Notification::new()
        .summary(title)
        .body(body)
        .urgency(urgency)
        .finalize()
        .show()
        .unwrap();
}

#[derive(PartialEq)]
enum UPowerBatteryState {
    PendingCharge,
    Charging,
    Discharging,
    FullyCharged
}

impl UPowerBatteryState {
    fn update(self, other: Self) -> Self {
        match other {
            Self::PendingCharge | Self::FullyCharged => self,
            Self::Charging | Self::Discharging => other
        }
    }
}

impl From<&str> for UPowerBatteryState {
    fn from(value: &str) -> Self {
        match value {
            "pending-charge" => Self::PendingCharge,
            "charging" => Self::Charging,
            "discharging" => Self::Discharging,
            "fully-charged" => Self::FullyCharged,
            _ => {
                dbg!(value);
                unreachable!()
            }
        }
    }
}

struct BatteryData {
    percentage: f32,
    hours_left: Option<f32>,
    discharging: bool
}

impl BatteryData {
    fn create_percentage_string(&self) -> String {
        format!("{}%", self.percentage.floor() as u32)
    }
    fn create_time_string(&self) -> Option<String> {
        let hours_left = self.hours_left?;
        let hours = hours_left.floor();
        let minutes = ((hours_left - hours) * 60.0).floor();

        let hours = hours as u32;
        let minutes = minutes as u32;

        let hour_str = (hours != 0).then(|| {
            if hours == 1 { format!("{hours} hour") }
            else { format!("{hours} hours") }
        });

        let conjunction = (hours != 0 && minutes != 0).then(|| " and ".to_string());

        let minute_str = (minutes != 0).then(|| {
            if minutes == 1 { format!("{minutes} minute") }
            else { format!("{minutes} minutes") }
        });

        Some([hour_str, conjunction, minute_str].into_iter().filter_map(|x| x).collect())
    }
    fn new() -> Self {
        let enumerate_command = Command::new("upower")
                .arg("-e")
                .output()
                .unwrap();
        let batteries = std::str::from_utf8(&enumerate_command.stdout).unwrap().lines()
            .filter(|s| !s.contains("hid"))
            .filter(|s| s.contains("battery"));

        let mut total_energy = 0.0;
        let mut total_energy_full = 0.0;
        let mut total_energy_rate = 0.0;
        let mut total_state = UPowerBatteryState::FullyCharged;

        for battery in batteries {
            let info_command = Command::new("upower")
                .arg("-i")
                .arg(battery)
                .output()
                .unwrap();
            let lines = std::str::from_utf8(&info_command.stdout).unwrap().lines()
                .filter(|s|
                    s.contains("energy:") ||
                    s.contains("energy-full:") ||
                    s.contains("energy-rate:") ||
                    s.contains("state:")
                );

            for line in lines {
                let mut words = line.split_whitespace();

                let name = words.next().unwrap();
                let data = words.next().unwrap();
                let data_float = data.parse::<f32>();

                match name {
                    "state:" => total_state = total_state.update(data.into()),
                    "energy:" => total_energy += data_float.unwrap(),
                    "energy-full:" => total_energy_full += data_float.unwrap(),
                    "energy-rate:" => total_energy_rate += data_float.unwrap(),
                    _ => unreachable!()
                }
            }
        }

        if total_state == UPowerBatteryState::FullyCharged {
            return BatteryData {
                percentage: 100.0,
                hours_left: None,
                discharging: false,
            };
        }
        
        let percentage = (total_energy / total_energy_full) * 100.0;
        let discharging = total_state == UPowerBatteryState::Discharging;
        let hours_left = (total_energy_rate != 0.0).then_some(
            if discharging { total_energy / total_energy_rate }
            else { (total_energy_full - total_energy) / total_energy_rate }
        );

        BatteryData {
            percentage,
            discharging,
            hours_left
        }
    }

    
}

#[derive(PartialEq)]
enum BatteryState {
    None,
    Charging,
    Normal,
    Low,
    Critical
}

fn send_battery_notification(
    title: &str, 
    verb: &str, 
    battery_data: &BatteryData, 
    urgency: Urgency 
) {
    let body = format!(
        "Battery is at {}. Will be {} in {}.",
        battery_data.create_percentage_string(),
        verb,
        battery_data.create_time_string().unwrap()
    );

    send_notification(title, &body, urgency);
}

impl BatteryState {
    fn notify(&self, battery_data: &BatteryData) {
        match *self {
            Self::None => unreachable!(),
            Self::Normal => 
                send_battery_notification(
                    "Battery Discharging", 
                    "empty", 
                    battery_data, 
                    Urgency::Normal
                ),
            Self::Low => 
                send_battery_notification(
                    "Battery Low", 
                    "empty", 
                    battery_data, 
                    Urgency::Critical
                ),
            Self::Critical => 
                send_battery_notification(
                    "Battery Very Low", 
                    "empty", 
                    battery_data, 
                    Urgency::Critical
                ),
            Self::Charging => 
                send_battery_notification(
                    "Battery Charging", 
                    "fully charged", 
                    battery_data, 
                    Urgency::Normal
                )
        }
    }

    fn new_state(&self, battery_data: &BatteryData) -> Option<Self> {
        let new_state =
            if !battery_data.discharging { BatteryState::Charging }
            else {
                if battery_data.percentage < 6.0 { BatteryState::Critical }
                else if battery_data.percentage < 16.0 { BatteryState::Low }
                else { BatteryState::Normal }
            };
        
        (*self != new_state).then_some(new_state)
    }

    fn to_class(&self) -> &'static str {
        match *self {
            Self::None => unreachable!(),
            Self::Charging => "charging",
            Self::Normal => "normal",
            Self::Low => "low",
            Self::Critical => "critical",
        }
    }
}

fn main() {
    let mut battery_state = BatteryState::None;
    let mut notified = true;

    for _ in BufReader::new(ChildProcess::new().child.stdout.take().unwrap()).lines() {
        let battery_data = BatteryData::new();
        if let Some(new_state) = battery_state.new_state(&battery_data) {
            battery_state = new_state;
            notified = false;
        }

        if !notified && battery_data.hours_left.is_some() {
            battery_state.notify(&battery_data);
            notified = true;
        }

        let tooltip = if let Some(s) = battery_data.create_time_string() {
            format!(
                "{} ({})", 
                battery_data.create_percentage_string(),
                s
            )
        } else {
            battery_data.create_percentage_string()
        };
        
        let icon = if battery_data.discharging { 
            discharging_icon_of_percentage(battery_data.percentage)
        } else {
            charging_icon_of_percentage(battery_data.percentage)
        };

        println!(
            "{{\"text\": \"{icon}\", \"class\": \"{}\", \"tooltip\": \"{tooltip}\"}}",
            battery_state.to_class()
        );
    }
}
