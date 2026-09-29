use std::{path::{Path, PathBuf}, process::{Child, Command, Stdio}};

use crate::{strategy::{Profile, Protocol, Strategy}, Result, SuZapretError};

pub trait Backend {
    fn start(&mut self, strategy: &Strategy) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&mut self) -> Result<bool>;
}

#[derive(Debug)]
pub struct ZapretBackend {
    root: PathBuf,
    child: Option<Child>,
}

impl ZapretBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), child: None }
    }

    fn resource_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() { path.to_owned() } else { self.root.join(path) }
    }

    fn command_for(&self, strategy: &Strategy) -> Result<Command> {
        #[cfg(windows)]
        let (program, working_dir) = (self.root.join("bin/winws.exe"), self.root.clone());
        #[cfg(not(windows))]
        let (program, working_dir) = (self.root.join("nfqws"), self.root.clone());

        let mut command = Command::new(program);
        command.current_dir(working_dir).stdout(Stdio::null()).stderr(Stdio::inherit());
        #[cfg(not(windows))]
        command.args(["--qnum=220", "--dpi-desync-fwmark=0x40000000"]);
        #[cfg(windows)]
        {
            command.arg(format!("--wf-tcp={}", strategy.intercept.tcp_ports.join(",")));
            command.arg(format!("--wf-udp={}", strategy.intercept.udp_ports.join(",")));
        }
        for (index, profile) in strategy.profiles.iter().enumerate() {
            self.append_profile(&mut command, profile, strategy)?;
            if index + 1 < strategy.profiles.len() {
                command.arg("--new");
            }
        }
        Ok(command)
    }

    fn append_profile(
        &self,
        command: &mut Command,
        profile: &Profile,
        strategy: &Strategy,
    ) -> Result<()> {
        let ports = profile
            .ports
            .iter()
            .filter_map(|port| match port.as_str() {
                "${game_filter_tcp}" if !profile.game_filter || !strategy.intercept.game_filter_tcp => None,
                "${game_filter_udp}" if !profile.game_filter || !strategy.intercept.game_filter_udp => None,
                "${game_filter_tcp}" | "${game_filter_udp}" => Some("1024-65535"),
                value => Some(value),
            })
            .collect::<Vec<_>>()
            .join(",");
        if ports.is_empty() {
            return Ok(());
        }
        match profile.protocol {
            Protocol::Tcp => { command.arg(format!("--filter-tcp={ports}")); }
            Protocol::Udp => { command.arg(format!("--filter-udp={ports}")); }
        }
        for path in &profile.hostlists {
            command.arg(format!("--hostlist={}", self.resource_path(path).display()));
        }
        for domain in &profile.hostlist_domains {
            command.arg(format!("--hostlist-domains={domain}"));
        }
        for path in &profile.ipsets {
            command.arg(format!("--ipset={}", self.resource_path(path).display()));
        }
        for path in &profile.ipset_excludes {
            command.arg(format!("--ipset-exclude={}", self.resource_path(path).display()));
        }
        command.args(&profile.arguments);
        Ok(())
    }

}

impl Backend for ZapretBackend {
    fn start(&mut self, strategy: &Strategy) -> Result<()> {
        if self.child.is_some() { return Err(SuZapretError::AlreadyRunning); }
        strategy.validate()?;
        #[cfg(not(windows))]
        setup_linux_firewall(strategy)?;
        let child = match self.command_for(strategy)?.spawn() {
            Ok(child) => child,
            Err(error) => {
                #[cfg(not(windows))]
                clear_linux_firewall()?;
                return Err(error.into());
            }
        };
        self.child = Some(child);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let Some(mut child) = self.child.take() else { return Err(SuZapretError::NotRunning); };
        let _ = child.kill();
        let _ = child.wait();
        #[cfg(not(windows))]
        clear_linux_firewall()?;
        Ok(())
    }

    fn is_running(&mut self) -> Result<bool> {
        let Some(child) = self.child.as_mut() else { return Ok(false); };
        Ok(child.try_wait()?.is_none())
    }
}

#[cfg(not(windows))]
fn setup_linux_firewall(strategy: &Strategy) -> Result<()> {
    run_nft(["add", "table", "inet", "suzapret"])?;
    run_nft([
        "add", "chain", "inet", "suzapret", "post", "{", "type", "filter",
        "hook", "postrouting", "priority", "mangle;", "}",
    ])?;
    run_nft([
        "add", "chain", "inet", "suzapret", "pre", "{", "type", "filter",
        "hook", "prerouting", "priority", "filter;", "}",
    ])?;

    let tcp_ports = strategy.intercept.tcp_ports.join(",");
    let udp_ports = strategy.intercept.udp_ports.join(",");
    if !tcp_ports.is_empty() {
        run_nft([
            "add", "rule", "inet", "suzapret", "post", "tcp", "dport", &format!("{{ {tcp_ports} }}"),
            "ct", "original", "packets", "1-6", "queue", "num", "220", "bypass",
        ])?;
        run_nft([
            "add", "rule", "inet", "suzapret", "pre", "tcp", "sport", &format!("{{ {tcp_ports} }}"),
            "ct", "reply", "packets", "1-3", "queue", "num", "220", "bypass",
        ])?;
    }
    if !udp_ports.is_empty() {
        run_nft([
            "add", "rule", "inet", "suzapret", "post", "udp", "dport", &format!("{{ {udp_ports} }}"),
            "ct", "original", "packets", "1-6", "queue", "num", "220", "bypass",
        ])?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn clear_linux_firewall() -> Result<()> {
    let _ = run_nft(["delete", "table", "inet", "suzapret"]);
    Ok(())
}

#[cfg(not(windows))]
fn run_nft<const N: usize>(args: [&str; N]) -> Result<()> {
    let mut command = Command::new("nft");
    command.args(args);
    let display = format!("{command:?}");
    let output = command.output()?;
    if output.status.success() { return Ok(()); }
    Err(SuZapretError::CommandFailed {
        command: display,
        status: output.status.to_string(),
        details: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Backend;

    #[test]
    fn new_backend_is_not_running() {
        let mut backend = ZapretBackend::new("runtime");
        assert_eq!(backend.is_running().ok(), Some(false));
    }

    #[test]
    fn stopping_an_idle_backend_returns_not_running() {
        let mut backend = ZapretBackend::new("runtime");
        assert!(matches!(backend.stop(), Err(SuZapretError::NotRunning)));
    }

    #[test]
    fn relative_resources_are_resolved_from_runtime_root() {
        let backend = ZapretBackend::new("runtime/linux");
        assert_eq!(backend.resource_path(Path::new("bin/nfqws")), PathBuf::from("runtime/linux/bin/nfqws"));
        assert_eq!(backend.resource_path(Path::new("/tmp/resource")), PathBuf::from("/tmp/resource"));
    }
}
