use std::fs;
use std::process::{Command, Stdio};

use urlencoding;

const FIELD_CODES: [&str; 14] = [
    "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m", "%",
];

pub const DEFAULT_SEARCH_URL_TEMPLATE: &str = "https://duckduckgo.com/?q={query}";

pub fn build_search_url(query: &str, template_override: Option<&str>) -> String {
    if let Some(template) = template_override {
        if let Some(url) = build_search_url_from_template(query, template) {
            return url;
        }
    }

    let encoded = urlencoding::encode(query).into_owned();
    DEFAULT_SEARCH_URL_TEMPLATE.replace("{query}", &encoded)
}

pub fn looks_like_url(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    if input.contains(char::is_whitespace) {
        return false;
    }

    let lower = input.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return true;
    }

    if lower.starts_with("www.") {
        return true;
    }

    if lower == "localhost" || lower.starts_with("localhost:") {
        return true;
    }

    let is_ipv4ish = lower
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ':');
    if is_ipv4ish && (lower.contains('.') || lower.contains(':')) {
        return true;
    }

    lower.contains('.') && !lower.starts_with('.')
}

pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return trimmed.to_string();
    }

    format!("https://{trimmed}")
}

pub fn open_default_browser(url: &str) -> bool {
    let output = Command::new("xdg-settings")
        .arg("get")
        .arg("default-web-browser")
        .output();

    let Ok(output) = output else {
        return webbrowser::open(url).is_ok();
    };

    let desktop = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if desktop.is_empty() {
        return webbrowser::open(url).is_ok();
    }

    let home = std::env::var("HOME").unwrap_or_default();

    let paths = [
        format!("{home}/.local/share/applications/{desktop}"),
        format!("/usr/share/applications/{desktop}"),
    ];

    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                if line.starts_with("Exec=") {
                    let exec = line.trim_start_matches("Exec=");
                    let parts = parse_exec(exec);

                    if parts.is_empty() {
                        continue;
                    }

                    let mut cmd = Command::new(&parts[0]);

                    if parts.len() > 1 {
                        cmd.args(&parts[1..]);
                    }

                    if !parts.iter().any(|p| p == "--new-window") {
                        cmd.arg("--new-window");
                    }

                    return cmd
                        .arg(url)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .is_ok();
                }
            }
        }
    }

    webbrowser::open(url).is_ok()
}

fn build_search_url_from_template(query: &str, template: &str) -> Option<String> {
    let encoded = urlencoding::encode(query).into_owned();

    if template.contains("{query}") {
        return Some(template.replace("{query}", &encoded));
    }

    if template.contains("%s") {
        return Some(template.replace("%s", &encoded));
    }

    Some(format!("{template}{encoded}"))
}

pub fn parse_exec(exec_line: &str) -> Vec<String> {
    let parts = match shlex::split(exec_line) {
        Some(parts) => parts,
        None => return Vec::new(),
    };

    parts
        .into_iter()
        .filter(|part| !FIELD_CODES.contains(&part.as_str()) && !part.starts_with('%'))
        .collect()
}
