use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let mut child = Command::new("rsync")
        .args(["-av", "--progress", "source/", "destination/"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_thread = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            println!("[rsync] {}", line.unwrap());
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            eprintln!("[rsync] {}", line.unwrap());
        }
    });

    let status = child.wait()?;

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    if !status.success() {
        return Err(std::io::Error::other(format!(
            "rsync failed with status: {}",
            status
        )));
    }

    println!("rsync completed successfully");
    Ok(())
}
