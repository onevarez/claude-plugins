//! `viewfinder` — Session orchestrator for the Viewfinder Claude Code plugin.
//!
//! Usage:
//!   viewfinder session init              # Create session, write active-session marker
//!   viewfinder session finalize <ID>     # WebM → cursor → transcode → zoom → compose via kineto
//!   viewfinder hook                      # Parse Claude Code hook payload from stdin

mod hooks;
mod zoom;

use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use chrono::Utc;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "viewfinder", version, about = "Cinematic browser session recordings")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    Hook,
}

#[derive(Subcommand)]
enum SessionAction {
    Init,
    Finalize { session_id: String },
}

fn main() {
    let vf_bin = vf_dir().join("bin");
    if let Ok(path) = std::env::var("PATH") {
        std::env::set_var("PATH", format!("{}:{}", vf_bin.display(), path));
    }

    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Session { action } => match action {
            SessionAction::Init => cmd_init(),
            SessionAction::Finalize { session_id } => cmd_finalize(&session_id),
        },
        Commands::Hook => cmd_hook(),
    };
    std::process::exit(code);
}

// ── session init ──

fn cmd_init() -> i32 {
    let dir = vf_dir();
    let session_id = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let session_dir = dir.join("sessions").join(&session_id);

    if let Err(e) = fs::create_dir_all(&session_dir) {
        eprintln!("ERROR: {}", e);
        return 1;
    }
    let _ = fs::write(dir.join("active-session"), &session_id);

    // Clean stale playwright artifacts from previous sessions
    clean_dir("playwright-output", "log");
    clean_dir("playwright-videos", "webm");

    println!("{}", session_id);
    0
}

// ── session finalize ──

fn cmd_finalize(session_id: &str) -> i32 {
    let dir = vf_dir();
    let bin_dir = dir.join("bin");
    let session_dir = dir.join("sessions").join(session_id);
    let output_dir = session_dir.join("output");
    let _ = fs::create_dir_all(&output_dir);

    // 1. Find WebM
    let webm = match find_latest("playwright-videos", "webm") {
        Some(p) => {
            let mb = fs::metadata(&p).map(|m| m.len()).unwrap_or(0) as f64 / 1_048_576.0;
            eprintln!("Video: {} ({:.1} MB)", p.display(), mb);
            p
        }
        None => {
            eprintln!("ERROR: No WebM in playwright-videos/");
            return 1;
        }
    };

    // 2. Extract cursor from console logs
    let cursor_path = session_dir.join("cursor.json");
    let cursor_count = extract_cursor(&cursor_path);
    if cursor_count > 0 {
        eprintln!("Cursor: {} samples", cursor_count);
    }

    // 3. Transcode WebM → MP4
    let mp4 = output_dir.join("session.mp4");
    let ffmpeg = match find_bin("ffmpeg", &bin_dir) {
        Some(f) => f,
        None => { eprintln!("ERROR: ffmpeg not found. Run /viewfinder:setup."); return 1; }
    };

    eprintln!("Transcoding...");
    let ok = Command::new(&ffmpeg)
        .args(["-i", &webm.to_string_lossy(), "-c:v", "libx264", "-preset", "ultrafast",
               "-crf", "18", "-pix_fmt", "yuv420p", "-y", &mp4.to_string_lossy()])
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    if !ok { eprintln!("ERROR: transcode failed"); return 1; }

    let mb = fs::metadata(&mp4).map(|m| m.len()).unwrap_or(0) as f64 / 1_048_576.0;
    eprintln!("  {:.1} MB", mb);

    // 4. Probe dimensions
    let (w, h, dur) = probe(&bin_dir, &mp4);

    // 5. Compute zoom segments from cursor clicks
    let zoom_file = output_dir.join("zoom_segments.json");
    let has_zoom = if cursor_count > 0 {
        let samples: Vec<zoom::CursorSample> = fs::read_to_string(&cursor_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let segments = zoom::compute_zoom_segments(&samples, w as f64, h as f64, 2.0);
        if !segments.is_empty() {
            eprintln!("Zoom: {} segments", segments.len());
            let _ = fs::write(&zoom_file, serde_json::to_string_pretty(&segments).unwrap_or_default());
            true
        } else {
            false
        }
    } else {
        false
    };

    // 6. Compose via kineto
    let cinematic = output_dir.join("cinematic.mp4");
    let kineto = find_bin("kineto", &bin_dir);

    if let Some(k) = kineto {
        let mut args = vec![
            "export", "-i", &mp4.to_string_lossy(), "-o", &cinematic.to_string_lossy(),
            "--bg-type", "color", "--bg-color", "#0f0f23",
            "--padding", "0.06", "--corner-radius", "16", "--shadow",
            "--codec", "h264", "--quality", "high",
        ].into_iter().map(String::from).collect::<Vec<_>>();

        if has_zoom {
            args.push("--zoom-segments-file".into());
            args.push(zoom_file.to_string_lossy().to_string());
        }

        eprintln!("Composing...");
        let ok = Command::new(&k).args(&args).status().map(|s| s.success()).unwrap_or(false);
        if !ok { eprintln!("ERROR: kineto failed"); return 1; }
    } else {
        eprintln!("WARNING: kineto not found — copying raw MP4");
        let _ = fs::copy(&mp4, &cinematic);
    }

    // 7. Cleanup and report
    let _ = fs::remove_file(&webm);

    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(&cinematic).status();
    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(&cinematic).status();

    let size = fs::metadata(&cinematic).map(|m| m.len()).unwrap_or(0) as f64 / 1_048_576.0;
    let tier = if cursor_count > 0 { "Tier 2" } else { "Tier 1" };
    eprintln!("{} — {:.1}s, {:.1} MB", tier, dur, size);
    println!("{}", cinematic.display());
    0
}

// ── hook ──

fn cmd_hook() -> i32 {
    let dir = vf_dir();
    let session_id = match fs::read_to_string(dir.join("active-session")) {
        Ok(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => return 0,
    };

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return 0;
    }

    let log_path = dir.join("sessions").join(&session_id).join("events.jsonl");
    if let Some(event) = hooks::parse_hook_payload(&input) {
        let _ = hooks::append_event(&log_path, &event);
    }

    0
}

// ── helpers ──

fn vf_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".viewfinder")
}

fn clean_dir(dir: &str, ext: &str) {
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.filter_map(|e| e.ok()) {
            if e.path().extension().map(|x| x == ext).unwrap_or(false) {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

fn find_latest(dir: &str, ext: &str) -> Option<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == ext).unwrap_or(false))
        .collect();
    entries.sort_by(|a, b| {
        let ma = a.metadata().and_then(|m| m.modified()).ok();
        let mb = b.metadata().and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });
    entries.first().map(|e| e.path())
}

fn find_bin(name: &str, bin_dir: &Path) -> Option<PathBuf> {
    let local = bin_dir.join(name);
    if local.exists() { return Some(local); }
    Command::new(name).arg("-version")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .status().ok().filter(|s| s.success())
        .map(|_| PathBuf::from(name))
}

fn extract_cursor(output_path: &Path) -> usize {
    let mut events: Vec<serde_json::Value> = Vec::new();
    let entries = match fs::read_dir("playwright-output") {
        Ok(e) => e, Err(_) => return 0,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if entry.path().extension().map(|e| e == "log").unwrap_or(false) {
            if let Ok(file) = fs::File::open(entry.path()) {
                for line in BufReader::new(file).lines().filter_map(|l| l.ok()) {
                    if let Some(idx) = line.find("[SO_CURSOR]") {
                        let json = line[idx + 11..].trim();
                        let json = json.rfind(" @ ").map(|i| &json[..i]).unwrap_or(json);
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                            events.push(v);
                        }
                    }
                }
            }
        }
    }
    let count = events.len();
    if count > 0 {
        if let Some(p) = output_path.parent() { let _ = fs::create_dir_all(p); }
        let _ = fs::write(output_path, serde_json::to_string(&events).unwrap_or_default());
    }
    count
}

fn probe(bin_dir: &Path, video: &Path) -> (u32, u32, f64) {
    let fp = match find_bin("ffprobe", bin_dir) { Some(f) => f, None => return (1920, 1080, 0.0) };
    let out = Command::new(&fp)
        .args(["-v", "quiet", "-print_format", "json", "-show_streams", "-show_format"])
        .arg(video).output().ok();
    let out = match out { Some(o) if o.status.success() => o, _ => return (1920, 1080, 0.0) };
    let j: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
    let vs = j.get("streams").and_then(|s| s.as_array())
        .and_then(|a| a.iter().find(|s| s.get("codec_type").and_then(|t| t.as_str()) == Some("video")));
    let w = vs.and_then(|s| s.get("width")).and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
    let h = vs.and_then(|s| s.get("height")).and_then(|v| v.as_u64()).unwrap_or(1080) as u32;
    let d = j.get("format").and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    (w, h, d)
}
