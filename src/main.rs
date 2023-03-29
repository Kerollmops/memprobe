use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use csv::Writer;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

/// A tool to probe the memory usage of a program
///
/// You can run this command on Linux:
///     memprobe $(pidof firefox)
///
/// Or this one on mac OS:
///     memprobe $(pgrep firefox)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The Process ID to measure the memory usage of
    pid: Pid,

    /// The interval, in milliseconds, to wait between two memory usage probings
    #[arg(long, default_value_t = 250)]
    interval_ms: u64,

    /// Print the CSV file to stdout instead of a file.
    #[arg(long, conflicts_with = "output_file")]
    stdout: bool,

    /// The file used to output the CSV memory usage data
    ///
    /// The default path is `./memprobe-$PID.csv`.
    #[arg(long)]
    output_file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let Args { pid, interval_ms, stdout, output_file } = Args::parse();

    let mut system = System::new();
    system.refresh_process(pid);

    let mut writer = writer_from_args(pid, stdout, output_file)
        .map(Writer::from_writer)
        .context("when creating the CSV file")?;

    writer.write_record(&["RES", "VIRT"]).context("when writing the headers into the CSV file")?;

    while system.refresh_process(pid) {
        if let Some(process) = system.process(pid) {
            let memory = process.memory();
            let virtual_memory = process.virtual_memory();
            writer
                .write_record(&[memory.to_string(), virtual_memory.to_string()])
                .context("when writing a new line into the CSV file")?;
            writer.flush().context("when flushing the CSV file")?;
            thread::sleep(Duration::from_millis(interval_ms));
        }
    }

    Ok(())
}

fn writer_from_args(
    pid: Pid,
    stdout: bool,
    output_file: Option<PathBuf>,
) -> anyhow::Result<Box<dyn Write>> {
    if stdout {
        Ok(Box::new(io::stdout()) as Box<dyn Write>)
    } else {
        let path = output_file.unwrap_or_else(|| PathBuf::from(format!("memprobe-{}.csv", pid)));
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .with_context(|| format!("trying to create and truncate `{}`", path.display()))?;
        Ok(Box::new(file) as _)
    }
}
