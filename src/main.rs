use chrono::{DateTime, Days, Duration, Local, NaiveDate};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize)]
struct TimeTracker {
    records: HashMap<String, Vec<(DateTime<Local>, Option<DateTime<Local>>)>>,
}

impl TimeTracker {
    fn new(file: &mut File) -> Result<Self, std::io::Error> {
        let mut data: String = String::new();
        file.read_to_string(&mut data)?;
        Ok(serde_json::from_str(&data).unwrap_or_else(|e| {
            println!("decode error {:?}", e);
            TimeTracker {
                records: HashMap::new(),
            }
        }))
    }

    fn start(&mut self, date: &str) {
        let entry = self.records.entry(date.to_string()).or_default();
        entry.push((Local::now(), None));
        println!("Started work at {}", entry.last().unwrap().0);
    }

    fn stop(&mut self, date: &str) {
        if let Some(entries) = self.records.get_mut(date) {
            if let Some(last) = entries.last_mut() {
                if last.1.is_none() {
                    last.1 = Some(Local::now());
                    println!("Stopped work at {}", last.1.unwrap());
                    let duration = last.1.unwrap() - last.0;
                    println!(
                        "Worked {} hours today.",
                        duration.num_minutes() as f64 / 60.0
                    );
                } else {
                    println!("Error: Already stopped.");
                }
            } else {
                println!("Error: No start entry found for today.");
            }
        } else {
            println!("Error: No entries found for this date.");
            let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
            match dt.checked_sub_days(Days::new(1)) {
                Some(yesterday) => self.stop(yesterday.to_string().as_str()),
                None => {
                    println!("Error: No start entry found for yesterday too.");
                }
            };
        }
    }
    fn fetch_latest(&self, date: &str) -> Option<&(DateTime<Local>, Option<DateTime<Local>>)> {
        if let Some(entries) = self.records.get(date) {
            entries.last().filter(|&last| last.1.is_none())
        } else {
            let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
            match dt.checked_sub_days(Days::new(1)) {
                Some(yesterday) => self.fetch_latest(yesterday.to_string().as_str()),
                None => None,
            }
        }
    }

    fn working(&self, date: &str) {
        if let Some(entry) = self.fetch_latest(date) {
            let now = Local::now();
            let duration: Duration = now - entry.0;
            let hours = duration.num_minutes() as f64 / 60.0;
            println!("current working {:.2} hours", hours);
        } else {
            println!("Not working now");
        }
    }

    fn summary(&self) {
        let mut month_summary: HashMap<String, Duration> = HashMap::new();
        for (date, entries) in &self.records {
            let month = date[..7].to_string(); // Extracts YYYY-MM from YYYY-MM-DD
            let total_duration: Duration = entries
                .iter()
                .map(|(start, end)| end.unwrap_or(Local::now()) - *start)
                .sum();
            let month_total = month_summary.entry(month).or_insert(Duration::zero());
            *month_total += total_duration;
        }

        for (month, duration) in month_summary {
            let hours = duration.num_minutes() as f64 / 60.0;

            println!(
                "{}: {} hours {} min",
                month,
                hours as i64,
                ((hours - (hours as i64) as f64) * 60.0) as i64
            );
        }
    }

    fn save(&mut self, file: &mut File) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(&self).expect("Failed to serialize data");
        file.write_all(json.as_bytes())
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    file_path: String,
}
#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
    Current,
    Summary,
}
fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut file = File::open(cli.file_path.clone())?;
    let mut tracker = TimeTracker::new(&mut file)?;

    let today = Local::now().format("%Y-%m-%d").to_string();

    match &cli.command {
        Commands::Start => {
            tracker.start(&today);
            let mut savefile = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(cli.file_path)?;
            tracker.save(&mut savefile)?
        }
        Commands::Stop => {
            tracker.stop(&today);
            let mut savefile = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(cli.file_path)?;
            tracker.save(&mut savefile)?
        }
        Commands::Current => {
            tracker.working(&today);
        }
        Commands::Summary => {
            tracker.summary();
        }
    }
    Ok(())
}
