use std::{fs, path::Path};

use crate::{strategy::{Intercept, Metadata, Profile, Protocol, Strategy}, Result, SuZapretError};

pub fn convert_bat(path: impl AsRef<Path>) -> Result<Strategy> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    let payload = source
        .replace('\r', "")
        .replace('^', "")
        .replace('\n', " ");
    let start = payload
        .to_ascii_lowercase()
        .find("winws.exe")
        .ok_or_else(|| invalid("winws.exe launch command not found"))?;
    let tokens = tokenize(&payload[start + "winws.exe".len()..]);
    let wf_tcp = value_after(&tokens, "--wf-tcp=")
        .ok_or_else(|| invalid("--wf-tcp is missing"))?;
    let wf_udp = value_after(&tokens, "--wf-udp=")
        .ok_or_else(|| invalid("--wf-udp is missing"))?;
    let mut profiles = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if token == "--new" {
            add_profile(&mut profiles, &current)?;
            current.clear();
        } else if !token.starts_with("--wf-") {
            current.push(token);
        }
    }
    add_profile(&mut profiles, &current)?;
    let id = path.file_stem().and_then(|value| value.to_str()).unwrap_or("strategy");
    let id = normalize_id(id);
    Ok(Strategy {
        metadata: Metadata { id, name: display_name(path), version: 1, source: String::new() },
        intercept: Intercept {
            tcp_ports: split_ports(&wf_tcp),
            udp_ports: split_ports(&wf_udp),
            game_filter_tcp: true,
            game_filter_udp: true,
        },
        profiles,
    })
}

pub fn write_toml(strategy: &Strategy, path: impl AsRef<Path>) -> Result<()> {
    strategy.validate()?;
    let text = toml::to_string_pretty(strategy)
        .map_err(|error| invalid(&format!("cannot serialize strategy: {error}")))?;
    fs::write(path, text)?;
    Ok(())
}

fn add_profile(profiles: &mut Vec<Profile>, tokens: &[String]) -> Result<()> {
    let Some((index, filter)) = tokens.iter().enumerate().find(|(_, value)| {
        value.starts_with("--filter-tcp=") || value.starts_with("--filter-udp=")
    }) else {
        return Ok(());
    };
    let (protocol, ports) = if let Some(value) = filter.strip_prefix("--filter-tcp=") {
        (Protocol::Tcp, value)
    } else {
        (Protocol::Udp, filter.trim_start_matches("--filter-udp="))
    };
    let ports = ports.split(',').map(|port| match port {
        "%GameFilterTCP%" | "%GameFilterUDP%" => match protocol {
            Protocol::Tcp => "${game_filter_tcp}".to_owned(),
            Protocol::Udp => "${game_filter_udp}".to_owned(),
        },
        value => value.to_owned(),
    }).collect::<Vec<_>>();
    let arguments = tokens.iter().enumerate()
        .filter(|(position, value)| *position != index && !value.starts_with("--wf-"))
        .map(|(_, value)| normalize_argument(value))
        .collect();
    let game_filter = ports.iter().any(|port| port.starts_with("${game_filter_"));
    profiles.push(Profile {
        id: format!("profile_{}", profiles.len() + 1),
        protocol,
        ports,
        game_filter,
        hostlists: Vec::new(),
        hostlist_domains: Vec::new(),
        ipsets: Vec::new(),
        ipset_excludes: Vec::new(),
        arguments,
    });
    Ok(())
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    for character in input.chars() {
        match character {
            '"' => quoted = !quoted,
            character if character.is_whitespace() && !quoted => {
                if !token.is_empty() { tokens.push(std::mem::take(&mut token)); }
            }
            character => token.push(character),
        }
    }
    if !token.is_empty() { tokens.push(token); }
    tokens
}

fn value_after(tokens: &[String], prefix: &str) -> Option<String> {
    tokens.iter().find_map(|token| token.strip_prefix(prefix).map(ToOwned::to_owned))
}

fn split_ports(value: &str) -> Vec<String> {
    value.split(',').filter(|port| !port.contains("GameFilter")).map(ToOwned::to_owned).collect()
}

fn normalize_argument(value: &str) -> String {
    value.replace("%BIN%", "bin/").replace("%LISTS%", "lists/").replace('\\', "/")
}

fn normalize_id(value: &str) -> String {
    value.chars().map(|character| if character.is_ascii_alphanumeric() { character.to_ascii_lowercase() } else { '_' }).collect()
}

fn display_name(path: &Path) -> String {
    let value = path.file_stem().and_then(|value| value.to_str()).unwrap_or("Strategy");
    let value = value.replace('_', " ");
    let value = value.replace('(', "").replace(')', "");
    let mut words = value.split_whitespace();
    let first = words.next().unwrap_or("Strategy");
    let suffix = words.collect::<Vec<_>>().join(" ");
    if suffix.is_empty() { return capitalize(first); }
    format!("{} ({})", capitalize(first), suffix.to_uppercase())
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map(|first| first.to_uppercase().collect::<String>() + characters.as_str()).unwrap_or_default()
}

fn invalid(message: &str) -> SuZapretError { SuZapretError::InvalidStrategy(message.to_owned()) }

#[cfg(test)]
mod tests {
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    use super::*;

    fn temporary_file() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("suzapret-converter-test-{suffix}.bat"))
    }

    #[test]
    fn converts_profiles_and_normalizes_paths() {
        let path = temporary_file();
        let source = r#"start "zapret" winws.exe --wf-tcp=80,443,%GameFilterTCP% --wf-udp=443,%GameFilterUDP% ^
--filter-tcp=443 --hostlist="%LISTS%list.txt" --dpi-desync=fake --new ^
--filter-udp=%GameFilterUDP% --dpi-desync=fake --dpi-desync-fake-quic="%BIN%quic.bin""#;
        assert!(fs::write(&path, source).is_ok());

        let strategy = convert_bat(&path);
        assert!(strategy.is_ok());
        let strategy = strategy.unwrap_or_else(|_| unreachable!());
        assert_eq!(strategy.intercept.tcp_ports, vec!["80", "443"]);
        assert_eq!(strategy.profiles.len(), 2);
        assert_eq!(strategy.profiles[0].arguments[0], "--hostlist=lists/list.txt");
        assert!(strategy.profiles[1].game_filter);
        assert_eq!(strategy.profiles[1].arguments[1], "--dpi-desync-fake-quic=bin/quic.bin");
        assert!(fs::remove_file(path).is_ok());
    }

    #[test]
    fn rejects_a_file_without_a_launch_command() {
        let path = temporary_file();
        assert!(fs::write(&path, "echo no zapret").is_ok());
        assert!(matches!(convert_bat(&path), Err(SuZapretError::InvalidStrategy(message)) if message.contains("winws.exe")));
        assert!(fs::remove_file(path).is_ok());
    }

    #[test]
    fn writes_valid_toml() {
        let input = temporary_file();
        let output = input.with_extension("toml");
        let source = "winws.exe --wf-tcp=443 --wf-udp=443 --filter-tcp=443 --dpi-desync=fake";
        assert!(fs::write(&input, source).is_ok());
        let strategy = convert_bat(&input);
        assert!(strategy.is_ok());
        let strategy = strategy.unwrap_or_else(|_| unreachable!());
        assert!(write_toml(&strategy, &output).is_ok());
        assert!(Strategy::from_file(&output).is_ok());
        assert!(fs::remove_file(input).is_ok());
        assert!(fs::remove_file(output).is_ok());
    }
}
