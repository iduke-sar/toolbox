use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use chrono::{Local, Utc, DateTime, Duration};
use clap::{App, Arg};
use std::collections::HashMap;
use rustyline::{DefaultEditor, Result};

fn get_timestamp() -> (String, String, DateTime<Local>) {
    let local_time = Local::now();
    let local_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let utc_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    (local_str, utc_str, local_time)
}

fn get_filename_timestamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}

struct LogEntry {
    local_time: String,
    utc_time: String,
    entry_type: String,
    description: String,
    tag: String,
}

fn parse_input(input: &str) -> (String, String, String) {
    let trimmed = input.trim();

    if trimmed.to_lowercase().starts_with("bug:") {
        ("OBSERVATION".to_string(), "BUG".to_string(), trimmed[4..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("warn:") {
        ("OBSERVATION".to_string(), "WARN".to_string(), trimmed[5..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("good:") {
        ("OBSERVATION".to_string(), "GOOD".to_string(), trimmed[5..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("pass:") {
        ("OBSERVATION".to_string(), "PASS".to_string(), trimmed[5..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("fail:") {
        ("OBSERVATION".to_string(), "FAIL".to_string(), trimmed[5..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("reload:") {
        ("RELOAD".to_string(), "VERSION".to_string(), trimmed[7..].trim().to_string())
    } else if trimmed.to_lowercase().starts_with("testcase:") {
        ("TEST CASE".to_string(), "NAME".to_string(), trimmed[9..].trim().to_string())
    } else {
        ("OBSERVATION".to_string(), "NOTE".to_string(), trimmed.to_string())
    }
}

fn write_logs_txt(
    logs: &Vec<LogEntry>,
    filepath: &PathBuf,
    test_name: &str,
    software_version: &str,
    test_objective: &str,
    test_operator: &str,
    participating_assets: &str,
    start_time: &str,
    end_time: &str,
    duration: &Duration,
    tag_counts: &HashMap<String, u32>,
) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(filepath)?;

    writeln!(file, "--- Test Session Started ---")?;
    writeln!(file, "Test Name            : {}", test_name)?;
    writeln!(file, "Software Version and Hash     : {}", software_version)?;
    writeln!(file, "Test Objective       : {}", test_objective)?;
    writeln!(file, "Test Operator        : {}", test_operator)?;
    writeln!(file, "Participating Asset(s) : {}", participating_assets)?;
    writeln!(file, "Start Time           : {}", start_time)?;
    writeln!(file, "---------------------------------------\n")?;

    writeln!(file, "--- Log Entries ---")?;

    for log in logs {
        let log_line = format!(
            "[Local: {}] [UTC: {}] [{}]\t[{}]\t{}",
            log.local_time,
            log.utc_time,
            log.entry_type.trim(),
            log.tag.trim(),
            log.description
        );
        writeln!(file, "{}", log_line)?;
    }

    writeln!(file, "\n--- Test Session Ended ---")?;
    writeln!(file, "End Time             : {}", end_time)?;
    writeln!(file, "Duration             : {} minutes {} seconds", duration.num_minutes(), duration.num_seconds() % 60)?;
    writeln!(file, "---------------------------------------")?;
    writeln!(file, "Summary of Tags:")?;
    for (tag, count) in tag_counts {
        writeln!(file, "{:<10}: {}", tag, count)?;
    }
    writeln!(file, "---------------------------------------")?;

    Ok(())
}

fn write_logs_csv(
    logs: &Vec<LogEntry>,
    filepath: &PathBuf,
    test_name: &str,
    software_version: &str,
    test_objective: &str,
    test_operator: &str,
    participating_assets: &str,
    start_time: &str,
    end_time: &str,
    duration: &Duration,
    tag_counts: &HashMap<String, u32>,
) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(filepath)?;

    writeln!(file, "Test Name,{}", test_name)?;
    writeln!(file, "Software Version,{}", software_version)?;
    writeln!(file, "Test Objective,{}", test_objective)?;
    writeln!(file, "Test Operator,{}", test_operator)?;
    writeln!(file, "Participating Asset(s),{}", participating_assets)?;
    writeln!(file, "Start Time,{}", start_time)?;
    writeln!(file)?;

    writeln!(file, "Local Time,UTC Time,Entry Type,Tag,Description")?;
    for log in logs {
        let log_line = format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            log.local_time,
            log.utc_time,
            log.entry_type.trim(),
            log.tag.trim(),
            log.description
        );
        writeln!(file, "{}", log_line)?;
    }

    writeln!(file)?;
    writeln!(file, "Summary Information")?;
    writeln!(file, "End Time,{}", end_time)?;
    writeln!(file, "Duration,{} minutes {} seconds", duration.num_minutes(), duration.num_seconds() % 60)?;
    writeln!(file)?;
    writeln!(file, "Tag,Count")?;

    for (tag, count) in tag_counts {
        writeln!(file, "{},{}", tag, count)?;
    }

    Ok(())
}

fn get_default_log_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let log_dir = exe_dir.join("logs");

    std::fs::create_dir_all(&log_dir).expect("Failed to create logs directory");

    log_dir
}

fn main() -> Result<()> {
    let matches = App::new("Test Logger")
        .version("1.0")
        .author("Isaac")
        .about("Logs test events with timestamps, entry type, and tags")
        .arg(Arg::with_name("log_dir")
            .short("d")
            .long("log-dir")
            .value_name("DIR")
            .help("Sets a custom log directory")
            .takes_value(true))
        .get_matches();

    let mut rl = DefaultEditor::new()?;

    let test_operator = rl.readline("Enter the test operator's name: ")?.trim().to_string();
    let test_name = rl.readline("Enter the test name: ")?.trim().to_string();
    let software_version = rl.readline("Enter the software version being tested: ")?.trim().to_string();
    let test_objective = rl.readline("Enter the test objective: ")?.trim().to_string();
    let participating_assets = rl.readline("Enter the participating asset(s): ")?.trim().to_string();

    let filename_timestamp = get_filename_timestamp();

    let log_dir = matches.value_of("log_dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| get_default_log_dir());

    let mut log_path_txt = log_dir.clone();
    let mut log_path_csv = log_dir.clone();

    let log_filename_txt = format!("{}_{}_log.txt", filename_timestamp, test_name.replace(" ", "_"));
    let log_filename_csv = format!("{}_{}_log.csv", filename_timestamp, test_name.replace(" ", "_"));

    log_path_txt.push(log_filename_txt);
    log_path_csv.push(log_filename_csv);

    println!("\n--- Log Files Will Be Saved To ---");
    println!("TXT Log Path : {}", log_path_txt.display());
    println!("CSV Log Path : {}", log_path_csv.display());
    println!("----------------------------------\n");

    let mut logs: Vec<LogEntry> = Vec::new();
    let mut tag_counts: HashMap<String, u32> = HashMap::new();

    let (start_local, _start_utc, start_dt) = get_timestamp();
    let start_time = start_local.clone();

    println!("--- Test Session Started ---");
    println!("Test Operator        : {}", test_operator);
    println!("Test Name            : {}", test_name);
    println!("Software Version     : {}", software_version);
    println!("Test Objective       : {}", test_objective);
    println!("Participating Asset(s) : {}", participating_assets);
    println!("Start Time           : {}", start_time);
    println!("---------------------------------------");
    println!("TAG OPTIONS :");
    println!("  bug:       <description>    - Log a BUG (critical issue)");
    println!("  warn:      <description>    - Log a WARN (potential issue)");
    println!("  good:      <description>    - Log a GOOD (confirmed correct behavior)");
    println!("  pass:      <description>    - Log a PASS (successful test case)");
    println!("  fail:      <description>    - Log a FAIL (failed test case)");
    println!("  reload:    <version>        - Log a RELOAD with new software version");
    println!("  testcase:  <name>           - Start a new TEST CASE");
    println!("  <plain description>         - Log a NOTE (general observation)");
    println!("---------------------------------------");
    println!("To END the test, type 'end' and press Enter.");
    println!("---------------------------------------\n");

    logs.push(LogEntry {
        local_time: start_local,
        utc_time: _start_utc,
        entry_type: "TEST START".to_string(),
        description: "Test started".to_string(),
        tag: "NOTE".to_string(),
    });

    loop {
        let input = rl.readline("> ")?;
        let behavior = input.trim();

        if behavior.is_empty() {
            continue;
        }

        if behavior.eq_ignore_ascii_case("end") {
            break;
        }

        let (entry_type, tag, desc) = parse_input(behavior);
        let (local, utc, _) = get_timestamp();

        logs.push(LogEntry {
            local_time: local,
            utc_time: utc,
            entry_type: entry_type.clone(),
            description: desc,
            tag: tag.clone(),
        });

        *tag_counts.entry(tag).or_insert(0) += 1;
    }

    let (end_local, _end_utc, end_dt) = get_timestamp();
    let elapsed = end_dt - start_dt;

    logs.push(LogEntry {
        local_time: end_local.clone(),
        utc_time: _end_utc,
        entry_type: "TEST END".to_string(),
        description: "Test session ended.".to_string(),
        tag: "NOTE".to_string(),
    });

    println!("\n--- Test Session Ended ---");
    println!("End Time             : {}", end_local);
    println!("Duration             : {} minutes {} seconds", elapsed.num_minutes(), elapsed.num_seconds() % 60);
    println!("---------------------------------------");
    println!("Summary of Tags:");
    for (tag, count) in &tag_counts {
        println!("{:<10}: {}", tag, count);
    }
    println!("---------------------------------------\n");

    write_logs_txt(
        &logs,
        &log_path_txt,
        &test_name,
        &software_version,
        &test_objective,
        &test_operator,
        &participating_assets,
        &start_time,
        &end_local,
        &elapsed,
        &tag_counts,
    )?;

    write_logs_csv(
        &logs,
        &log_path_csv,
        &test_name,
        &software_version,
        &test_objective,
        &test_operator,
        &participating_assets,
        &start_time,
        &end_local,
        &elapsed,
        &tag_counts,
    )?;

    println!("Logs successfully saved!");
    println!("TXT Log Path : {}", log_path_txt.display());
    println!("CSV Log Path : {}", log_path_csv.display());

    Ok(())
}
