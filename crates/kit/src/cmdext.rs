use std::process::{Command, Output, Stdio};

pub trait CommandRunExt {
    fn run(&mut self) -> std::io::Result<()>;
    fn run_and_parse_json<T: serde::de::DeserializeOwned>(&mut self) -> std::io::Result<T>;
}

impl CommandRunExt for Command {
    fn run(&mut self) -> std::io::Result<()> {
        let status = self.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("command exited with status {status}"),
            ))
        }
    }

    fn run_and_parse_json<T: serde::de::DeserializeOwned>(&mut self) -> std::io::Result<T> {
        let out: Output = self
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()?;
        if !out.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("command exited with status {}", out.status),
            ));
        }
        serde_json::from_slice(&out.stdout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
