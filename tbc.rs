use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const SONG_URL: &str = "https://www.youtube.com/watch?v=QDia3e12czc";
const MARKER: &str = "# tbc-texivi-birthday";

fn glyph(c: char) -> [&'static str; 5] {
    match c {
        'H' => ["█   █", "█   █", "█████", "█   █", "█   █"],
        'A' => [" ███ ", "█   █", "█████", "█   █", "█   █"],
        'P' => ["████ ", "█   █", "████ ", "█    ", "█    "],
        'Y' => ["█   █", " █ █ ", "  █  ", "  █  ", "  █  "],
        'B' => ["████ ", "█   █", "████ ", "█   █", "████ "],
        'I' => ["█████", "  █  ", "  █  ", "  █  ", "█████"],
        'R' => ["████ ", "█   █", "████ ", "█  █ ", "█   █"],
        'T' => ["█████", "  █  ", "  █  ", "  █  ", "  █  "],
        'D' => ["████ ", "█   █", "█   █", "█   █", "████ "],
        'E' => ["█████", "█    ", "████ ", "█    ", "█████"],
        'X' => ["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"],
        'V' => ["█   █", "█   █", "█   █", " █ █ ", "  █  "],
        _ => ["     ", "     ", "     ", "     ", "     "],
    }
}

fn big(word: &str, color: u8) {
    for row in 0..5 {
        let line: Vec<&str> = word.chars().map(|c| glyph(c)[row]).collect();
        println!("  \x1b[1;{}m{}\x1b[0m", color, line.join(" "));
    }
    println!();
}

fn open_url(url: &str) {
    Command::new("xdg-open")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
}

fn show() {
    print!("\x1b[2J\x1b[H\n");
    big("HAPPY", 91);
    big("BIRTHDAY", 93);
    big("TEXIVI", 95);
    open_url(SONG_URL);
    print!("  press enter to close...");
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}

fn has(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {} >/dev/null 2>&1", cmd)])
        .status()
        .unwrap()
        .success()
}

fn home() -> String {
    env::var("HOME").unwrap()
}

fn date(fmt: &str) -> String {
    let out = Command::new("date").arg(fmt).output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn is_may_30() -> bool {
    date("+%m%d") == "0530"
}

fn fire() {
    let exe = env::current_exe().unwrap().to_string_lossy().to_string();

    let mut terms: Vec<String> = vec![];
    if let Ok(t) = env::var("TERMINAL") {
        terms.push(t);
    }
    for t in [
        "kitty", "alacritty", "foot", "wezterm", "gnome-terminal", "konsole",
        "xfce4-terminal", "x-terminal-emulator", "xterm",
    ] {
        terms.push(t.to_string());
    }

    let t = terms.into_iter().find(|t| has(t)).expect("no terminal");
    let mut c = Command::new(&t);
    match t.as_str() {
        "gnome-terminal" => c.args(["--", &exe, "show"]),
        "wezterm" => c.args(["start", "--", &exe, "show"]),
        _ => c.args(["-e", &exe, "show"]),
    };
    if env::var("DISPLAY").is_err() && env::var("WAYLAND_DISPLAY").is_err() {
        c.env("DISPLAY", ":0");
    }
    c.spawn().unwrap();
}

fn fire_once() {
    let dir = format!("{}/.local/state/tbc", home());
    let file = format!("{}/fired", dir);
    let year = date("+%Y");
    if fs::read_to_string(&file).map(|s| s == year).unwrap_or(false) {
        return;
    }
    fire();
    fs::create_dir_all(&dir).unwrap();
    fs::write(&file, &year).unwrap();
}

fn daemon() {
    if has("pgrep") {
        let out = Command::new("pgrep").args(["-f", "(^|/)tbc daemon$"]).output().unwrap();
        let me = std::process::id().to_string();
        if String::from_utf8_lossy(&out.stdout).lines().any(|p| p != me) {
            return;
        }
    }
    loop {
        if is_may_30() {
            fire_once();
        }
        thread::sleep(Duration::from_secs(30));
    }
}

fn systemd_available() -> bool {
    has("systemctl")
        && Command::new("systemctl")
            .args(["--user", "show-environment"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
}

fn systemctl(args: &[&str]) {
    Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()
        .unwrap();
}

fn unit_dir() -> String {
    format!("{}/.config/systemd/user", home())
}

fn session_env() -> Vec<(String, String)> {
    ["DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR"]
        .iter()
        .filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect()
}

fn install_systemd(exe: &str) {
    let dir = unit_dir();
    fs::create_dir_all(&dir).unwrap();
    let envs: String = session_env()
        .iter()
        .map(|(k, v)| format!("Environment=\"{}={}\"\n", k, v))
        .collect();
    fs::write(
        format!("{}/tbc.service", dir),
        format!(
            "[Unit]\nDescription=tbc\n\n[Service]\nType=oneshot\nKillMode=process\n{}ExecStart={} fire\n",
            envs, exe
        ),
    )
    .unwrap();
    fs::write(
        format!("{}/tbc.timer", dir),
        "[Unit]\nDescription=tbc\n\n[Timer]\nOnCalendar=*-05-30 00:00:00\nPersistent=true\n\n[Install]\nWantedBy=timers.target\n",
    )
    .unwrap();
    systemctl(&["daemon-reload"]);
    systemctl(&["enable", "--now", "tbc.timer"]);
}

fn read_crontab() -> String {
    let out = Command::new("crontab").arg("-l").output().unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn write_crontab(content: &str) {
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(content.as_bytes()).unwrap();
    child.wait().unwrap();
}

fn without_ours(tab: &str) -> String {
    tab.lines()
        .filter(|l| !l.contains(MARKER))
        .map(|l| format!("{}\n", l))
        .collect()
}

fn install_cron(exe: &str) {
    let envs: String = session_env()
        .iter()
        .map(|(k, v)| format!("{}={} ", k, v))
        .collect();
    let line = format!("0 0 30 5 * {}{} fire {}\n", envs, exe, MARKER);
    write_crontab(&format!("{}{}", without_ours(&read_crontab()), line));
}

fn install_daemon(exe: &str) {
    let home = home();
    let profile = format!("{}/.profile", home);
    let old = fs::read_to_string(&profile).unwrap_or_default();
    let line = format!("{} daemon >/dev/null 2>&1 & {}\n", exe, MARKER);
    fs::write(&profile, format!("{}{}", without_ours(&old), line)).unwrap();

    let auto = format!("{}/.config/autostart", home);
    fs::create_dir_all(&auto).unwrap();
    fs::write(
        format!("{}/tbc.desktop", auto),
        format!("[Desktop Entry]\nType=Application\nName=tbc\nExec={} daemon\n", exe),
    )
    .unwrap();

    Command::new(exe)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
}

fn install() {
    print!("be reminded of texivis birthday? [y/N] ");
    io::stdout().flush().unwrap();
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).unwrap();
    if !ans.trim().to_lowercase().starts_with('y') {
        return;
    }

    let exe = env::current_exe().unwrap().to_string_lossy().to_string();
    if systemd_available() {
        install_systemd(&exe);
    } else if has("crontab") {
        install_cron(&exe);
    } else {
        install_daemon(&exe);
    }
    println!("done, see you on may 30th.");
}

fn uninstall() {
    if systemd_available() {
        systemctl(&["disable", "--now", "tbc.timer"]);
        let dir = unit_dir();
        let _ = fs::remove_file(format!("{}/tbc.timer", dir));
        let _ = fs::remove_file(format!("{}/tbc.service", dir));
        systemctl(&["daemon-reload"]);
    }
    if has("crontab") {
        write_crontab(&without_ours(&read_crontab()));
    }
    let profile = format!("{}/.profile", home());
    if let Ok(old) = fs::read_to_string(&profile) {
        fs::write(&profile, without_ours(&old)).unwrap();
    }
    let _ = fs::remove_file(format!("{}/.config/autostart/tbc.desktop", home()));
    if has("pkill") {
        Command::new("pkill").args(["-f", "(^|/)tbc daemon$"]).status().unwrap();
    }
    println!("removed.");
}

fn main() {
    match env::args().nth(1).as_deref() {
        Some("show") if is_may_30() => show(),
        Some("fire") if is_may_30() => fire_once(),
        Some("show") | Some("fire") => {}
        Some("daemon") => daemon(),
        Some("uninstall") => uninstall(),
        _ => install(),
    }
}
