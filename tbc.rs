use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

const SONG_URL: &str = "https://www.youtube.com/watch?v=BDb0UP9JzGM";
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
    c.spawn().unwrap();
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

fn install() {
    print!("be reminded of texivis birthday? [y/N] ");
    io::stdout().flush().unwrap();
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).unwrap();
    if !ans.trim().to_lowercase().starts_with('y') {
        return;
    }

    let exe = env::current_exe().unwrap();
    let out = Command::new("id").arg("-u").output().unwrap();
    let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();

    let line = format!(
        "0 0 30 5 * DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/{} WAYLAND_DISPLAY=wayland-0 {} fire {}\n",
        uid,
        exe.display(),
        MARKER
    );

    write_crontab(&format!("{}{}", without_ours(&read_crontab()), line));
    println!("done, see you on may 30th.");
}

fn uninstall() {
    write_crontab(&without_ours(&read_crontab()));
    println!("removed.");
}

fn is_may_30() -> bool {
    let out = Command::new("date").arg("+%m%d").output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim() == "0530"
}

fn main() {
    match env::args().nth(1).as_deref() {
        Some("show") if is_may_30() => show(),
        Some("fire") if is_may_30() => fire(),
        Some("show") | Some("fire") => {}
        Some("uninstall") => uninstall(),
        _ => install(),
    }
}
