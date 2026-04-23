use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    System,
}

impl Scope {
    pub fn flag(&self) -> &'static str {
        match self {
            Scope::User => "--user",
            Scope::System => "--system",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Scope::User => "USER",
            Scope::System => "SYSTEM",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
    Reload,
    Mask,
    Unmask,
}

impl Action {
    pub fn all() -> &'static [Action] {
        &[
            Action::Start,
            Action::Stop,
            Action::Restart,
            Action::Enable,
            Action::Disable,
            Action::Reload,
            Action::Mask,
            Action::Unmask,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Action::Start => "Start",
            Action::Stop => "Stop",
            Action::Restart => "Restart",
            Action::Enable => "Enable",
            Action::Disable => "Disable",
            Action::Reload => "Reload",
            Action::Mask => "Mask",
            Action::Unmask => "Unmask",
        }
    }

    fn systemctl_verb(&self) -> &'static str {
        match self {
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Restart => "restart",
            Action::Enable => "enable",
            Action::Disable => "disable",
            Action::Reload => "reload",
            Action::Mask => "mask",
            Action::Unmask => "unmask",
        }
    }

    fn needs_privilege(&self) -> bool {
        matches!(
            self,
            Action::Enable | Action::Disable | Action::Mask | Action::Unmask
        )
    }
}

#[derive(Debug, Clone)]
pub struct RawUnit {
    pub unit: String,
    pub load: String,
    pub active: String,
    pub sub: String,
    pub description: String,
}

pub fn list_units(scope: Scope) -> Result<Vec<RawUnit>> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--all",
            "--no-pager",
            "--no-legend",
            "--plain",
            scope.flag(),
        ])
        .output()
        .context("failed to run systemctl")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let units = stdout
        .lines()
        .filter_map(parse_unit_line)
        .collect();

    Ok(units)
}

fn parse_unit_line(line: &str) -> Option<RawUnit> {
    // systemctl --plain output: columns separated by whitespace
    // UNIT  LOAD  ACTIVE  SUB  DESCRIPTION
    let mut cols = line.split_whitespace();
    let unit = cols.next()?.to_string();
    // skip bullet/marker prefix if present (e.g. "●")
    let (unit, load) = if unit.starts_with('●') || unit.chars().next()?.is_ascii_punctuation() {
        let load = cols.next()?.to_string();
        (cols.next().unwrap_or(&unit).to_string(), load)
    } else {
        let load = cols.next()?.to_string();
        (unit, load)
    };
    let active = cols.next()?.to_string();
    let sub = cols.next()?.to_string();
    let description = cols.collect::<Vec<_>>().join(" ");

    Some(RawUnit { unit, load, active, sub, description })
}

pub fn unit_status(name: &str, scope: Scope) -> Result<String> {
    let output = Command::new("systemctl")
        .args(["status", "--no-pager", "-l", scope.flag(), name])
        .output()
        .context("failed to run systemctl status")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn journal_tail(name: &str, scope: Scope, lines: usize) -> Result<Vec<String>> {
    let mut args = vec![
        "-u".to_string(),
        name.to_string(),
        "-n".to_string(),
        lines.to_string(),
        "--no-pager".to_string(),
        "--output=short-iso".to_string(),
    ];
    if scope == Scope::User {
        args.push("--user".to_string());
    }

    let output = Command::new("journalctl")
        .args(&args)
        .output()
        .context("failed to run journalctl")?;

    let lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    Ok(lines)
}

pub fn do_action(action: &Action, name: &str, scope: Scope) -> Result<String> {
    let verb = action.systemctl_verb();

    let mut cmd = if action.needs_privilege() && scope == Scope::System {
        let mut c = Command::new("pkexec");
        c.args(["systemctl", verb, "--system", name]);
        c
    } else {
        let mut c = Command::new("systemctl");
        c.args([verb, scope.flag(), name]);
        c
    };

    let output = cmd.output().context("failed to run systemctl action")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        let msg = if stderr.is_empty() { stdout } else { stderr };
        anyhow::bail!("{}", msg.trim());
    }

    Ok(stdout)
}
