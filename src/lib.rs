use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub const VERSION: &str = "0.1.0-dev.1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub spa: bool,
    pub cors: bool,
    pub open_browser: bool,
    pub quiet: bool,
    pub bridge: Option<String>,
    pub auth: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Run(Config),
    Version,
    Help,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

/// Parses CLI arguments into an Action
pub fn parse_args<I>(args: I) -> Result<Action, String>
where
    I: IntoIterator<Item = String>,
{
    let mut port: Option<u16> = None;
    let mut host: Option<String> = None;
    let mut spa = false;
    let mut cors = false;
    let mut open_browser = false;
    let mut quiet = false;
    let mut bridge: Option<String> = None;
    let mut auth: Option<String> = None;
    let mut dir: Option<PathBuf> = None;

    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "-v" | "--version" => return Ok(Action::Version),
            "-l" | "--localhost" => host = Some("127.0.0.1".to_string()),
            "-s" | "--spa" => spa = true,
            "--cors" => cors = true,
            "-o" | "--open" => open_browser = true,
            "-q" | "--quiet" => quiet = true,
            "--bridge" => {
                let val = iter
                    .next()
                    .ok_or("Option '--bridge' requires a command binary argument")?;
                bridge = Some(val);
            }
            "--auth" => {
                let val = iter
                    .next()
                    .ok_or("Option '--auth' requires a token argument")?;
                auth = Some(val);
            }
            "-p" | "--port" => {
                let val = iter
                    .next()
                    .ok_or("Option '-p/--port' requires a port number argument")?;
                let p: u16 = val
                    .parse()
                    .map_err(|_| format!("Invalid port number: '{}'", val))?;
                port = Some(p);
            }
            "--host" => {
                let val = iter
                    .next()
                    .ok_or("Option '--host' requires an IP address argument")?;
                host = Some(val);
            }
            other => {
                if let Some(stripped) = other.strip_prefix("-p") {
                    if !stripped.is_empty() {
                        let p: u16 = stripped
                            .parse()
                            .map_err(|_| format!("Invalid port number: '{}'", stripped))?;
                        port = Some(p);
                        continue;
                    }
                }
                if other.starts_with('-') {
                    return Err(format!(
                        "Unknown option: '{}'. Use --help for usage instructions.",
                        other
                    ));
                }
                if dir.is_none() {
                    dir = Some(PathBuf::from(other));
                } else {
                    return Err(format!("Unexpected argument: '{}'", other));
                }
            }
        }
    }

    let host = host.unwrap_or_else(|| "0.0.0.0".to_string());
    let requested_port = port.unwrap_or(8080);

    // Auto-detect directory if none was provided
    let dir = match dir {
        Some(d) => d,
        None => {
            if Path::new("site").is_dir() {
                PathBuf::from("site")
            } else if Path::new("dist").is_dir() {
                PathBuf::from("dist")
            } else if Path::new("public").is_dir() {
                PathBuf::from("public")
            } else {
                PathBuf::from(".")
            }
        }
    };

    Ok(Action::Run(Config {
        dir,
        host,
        port: requested_port,
        spa,
        cors,
        open_browser,
        quiet,
        bridge,
        auth,
    }))
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct BridgeRequest {
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub stdin: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BridgeResponse {
    pub ok: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn verify_auth(
    auth_header: Option<&str>,
    x_token_header: Option<&str>,
    expected_token: &str,
) -> bool {
    let _ = (auth_header, x_token_header, expected_token);
    // Failing stub for TDD
    false
}

pub fn execute_bridge_command(
    binary: &str,
    args: &[String],
    input: Option<&str>,
) -> Result<BridgeResponse, String> {
    let _ = (binary, args, input);
    // Failing stub for TDD
    Err("Not implemented".to_string())
}

/// Finds an available port starting from `start_port` up to `start_port + 100`
pub fn find_available_port(host: &str, start_port: u16) -> Result<u16, String> {
    for port in start_port..=start_port.saturating_add(100) {
        if let Ok(listener) = TcpListener::bind((host, port)) {
            drop(listener);
            return Ok(port);
        }
    }
    Err(format!(
        "No available port found in range {}..{} on host {}",
        start_port,
        start_port + 100,
        host
    ))
}

/// Guesses MIME content-type based on file extension
pub fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        // Web & Core
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "wasm" => "application/wasm",
        "xml" => "application/xml",
        "webmanifest" | "manifest" => "application/manifest+json",

        // Audio
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "m4a" | "aac" => "audio/aac",
        "weba" => "audio/webm",
        "mid" | "midi" => "audio/midi",

        // Video
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",

        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",

        // Images
        "svg" | "svgz" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",

        // Documents
        "pdf" => "application/pdf",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",

        _ => "application/octet-stream",
    }
}

/// Decodes percent-encoded URL strings (%20 -> ' ', etc.)
pub fn percent_decode(input: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next()?;
            let h2 = chars.next()?;
            let hex_bytes = [h1, h2];
            let hex_str = std::str::from_utf8(&hex_bytes).ok()?;
            let val = u8::from_str_radix(hex_str, 16).ok()?;
            bytes.push(val);
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).ok()
}

/// Resolves URL path against base directory safely, preventing Directory Traversal attacks
pub fn resolve_path(base: &Path, url_path: &str) -> Option<PathBuf> {
    let clean = url_path.split('?').next().unwrap_or(url_path);
    let clean = clean.split('#').next().unwrap_or(clean);

    let decoded = percent_decode(clean)?;
    let relative = decoded.trim_start_matches('/');

    let mut target = base.to_path_buf();
    for comp in Path::new(relative).components() {
        match comp {
            Component::Normal(c) => target.push(c),
            Component::CurDir => {}
            _ => return None, // Reject ParentDir or RootDir escaping
        }
    }

    if target.is_dir() {
        let index = target.join("index.html");
        if index.is_file() {
            return Some(index);
        }
    }

    if target.is_file() {
        Some(target)
    } else {
        None
    }
}

/// Parses RFC 7233 HTTP Range header: bytes=start-end
pub fn parse_range_header(header: &str, file_size: u64) -> Option<ByteRange> {
    if !header.starts_with("bytes=") || file_size == 0 {
        return None;
    }
    let spec = &header["bytes=".len()..];
    let first = spec.split(',').next()?.trim();
    let parts: Vec<&str> = first.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let (s_str, e_str) = (parts[0].trim(), parts[1].trim());

    if s_str.is_empty() && !e_str.is_empty() {
        let suffix_len: u64 = e_str.parse().ok()?;
        let start = file_size.saturating_sub(suffix_len);
        Some(ByteRange {
            start,
            end: file_size - 1,
        })
    } else if !s_str.is_empty() && e_str.is_empty() {
        let start: u64 = s_str.parse().ok()?;
        if start >= file_size {
            return None;
        }
        Some(ByteRange {
            start,
            end: file_size - 1,
        })
    } else if !s_str.is_empty() && !e_str.is_empty() {
        let start: u64 = s_str.parse().ok()?;
        let end: u64 = e_str.parse().ok()?;
        if start > end || start >= file_size {
            return None;
        }
        let end = end.min(file_size - 1);
        Some(ByteRange { start, end })
    } else {
        None
    }
}

/// Prints Gosplan Radio Mayak terminal banner
pub fn print_banner(host: &str, port: u16, dir: &Path, spa: bool) {
    let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let dir_display = canonical.display();

    let local_url = format!("http://localhost:{}", port);
    let lan_url = if host == "0.0.0.0" {
        format!("http://0.0.0.0:{}", port)
    } else {
        format!("http://{}:{}", host, port)
    };

    println!();
    println!("       ★  РАДИОСТАНЦИЯ «МАЯК» (PORT {})  ★", port);
    println!("     ┌──────────────────────────────────────────────────┐");
    println!("     │   MAYAK STATIC BROADCAST & GOSPLAN TRANSMITTER   │");
    println!("     │   \"Broadcasting socialist culture to the Union\"  │");
    println!("     └──────────────────────────────────────────────────┘");
    println!("  📻 Local:      {}", local_url);
    if host == "0.0.0.0" {
        println!(
            "  📡 Network:    {} (all interfaces / LAN / Tailscale)",
            lan_url
        );
    }
    println!("  📂 Serving:    {}", dir_display);
    if spa {
        println!("  ⚙️  SPA Mode:   Active (404 -> index.html)");
    }
    println!("  ⌨️  Shutdown:   Press Ctrl+C to terminate transmission");
    println!();
}

/// Prints CLI usage information
pub fn print_help(binary_name: &str) {
    println!(
        r#"Mayak ({bin}) v{ver} — Gosplan Static HTTP Server

Usage:
  {bin} [options] [directory]

Options:
  -p, --port <number>     Initial port to bind (default: 8080, auto-increments if busy)
  --host <ip>             Host address to bind to (default: 0.0.0.0)
  -l, --localhost         Bind to 127.0.0.1 (restricts access to loopback only)
  -s, --spa               Single Page Application mode: fallback 404 to index.html
  --cors                  Send 'Access-Control-Allow-Origin: *' headers
  -o, --open              Open default web browser automatically
  -q, --quiet             Quiet mode (suppress per-request access logs)
  -h, --help              Display this help message
  -v, --version           Display version information

Examples:
  {bin}                           # Serve ./site, ./dist, or ./ on 0.0.0.0:8080
  {bin} site/ --open              # Serve site/ and launch browser
  {bin} -p 3000 --spa --cors      # Serve SPA on port 3000 with CORS headers
  {bin} --localhost               # Restrict access to local machine only
"#,
        bin = binary_name,
        ver = VERSION
    );
}

/// Executes the static web server
pub fn execute(mut config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve free port
    let actual_port = find_available_port(&config.host, config.port)
        .map_err(|e| format!("Port binding error: {}", e))?;

    if actual_port != config.port && !config.quiet {
        eprintln!(
            "Notice: Port {} was busy, automatically switched to port {}.",
            config.port, actual_port
        );
    }
    config.port = actual_port;

    // 2. Bind tiny_http Server
    let bind_addr = format!("{}:{}", config.host, config.port);
    let server = Server::http(&bind_addr)
        .map_err(|e| format!("Failed to bind server on {}: {}", bind_addr, e))?;

    // 3. Print Banner
    if !config.quiet {
        print_banner(&config.host, config.port, &config.dir, config.spa);
    }

    // 4. Open browser if requested
    if config.open_browser {
        let url = format!("http://localhost:{}", config.port);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            #[cfg(target_os = "macos")]
            let _ = Command::new("open").arg(&url).status();
            #[cfg(target_os = "linux")]
            let _ = Command::new("xdg-open").arg(&url).status();
            #[cfg(target_os = "windows")]
            let _ = Command::new("cmd").args(["/c", "start", &url]).status();
        });
    }

    // 5. Setup atomic shutdown signal
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_setup(r);

    let base_dir = config.dir.clone();
    let is_spa = config.spa;
    let is_cors = config.cors;
    let is_quiet = config.quiet;

    // 6. Request processing loop
    while running.load(Ordering::SeqCst) {
        let request = match server.recv() {
            Ok(rq) => rq,
            Err(e) => {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if !is_quiet {
                    eprintln!("Network receive error: {}", e);
                }
                continue;
            }
        };

        let method = request.method().clone();
        let url = request.url().to_string();

        if method != Method::Get && method != Method::Head {
            let res = Response::from_string("Method Not Allowed").with_status_code(StatusCode(405));
            let _ = request.respond(res);
            continue;
        }

        // Path resolution
        let resolved = resolve_path(&base_dir, &url).or_else(|| {
            if is_spa {
                let fallback = base_dir.join("index.html");
                if fallback.is_file() {
                    Some(fallback)
                } else {
                    None
                }
            } else {
                None
            }
        });

        let target_path = match resolved {
            Some(p) => p,
            None => {
                if !is_quiet {
                    eprintln!("404 Not Found: {}", url);
                }
                let res = Response::from_string("404 Not Found").with_status_code(StatusCode(404));
                let _ = request.respond(res);
                continue;
            }
        };

        // File metadata & range handling
        let metadata = match target_path.metadata() {
            Ok(m) => m,
            Err(_) => {
                let res = Response::from_string("500 Internal Server Error")
                    .with_status_code(StatusCode(500));
                let _ = request.respond(res);
                continue;
            }
        };

        let total_size = metadata.len();
        let mime = guess_mime(&target_path);

        // Handle HEAD method early with empty response
        if method == Method::Head {
            let mut resp = Response::empty(StatusCode(200));
            resp.add_header(
                Header::from_bytes(&b"Content-Length"[..], total_size.to_string().as_bytes())
                    .unwrap(),
            );
            resp.add_header(Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap());
            resp.add_header(Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).unwrap());
            resp.add_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap());
            if is_cors {
                resp.add_header(
                    Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
                );
            }
            let _ = request.respond(resp);
            if !is_quiet {
                println!("200 OK (HEAD): {} [{}] ({} bytes)", url, mime, total_size);
            }
            continue;
        }

        // Check for Range header
        let range_header = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Range"))
            .map(|h| h.value.as_str());

        let range_spec = range_header.and_then(|h| parse_range_header(h, total_size));

        if let Some(range) = range_spec {
            // 206 Partial Content
            let slice_len = range.end - range.start + 1;
            let mut file = match File::open(&target_path) {
                Ok(f) => f,
                Err(_) => {
                    let _ = request.respond(
                        Response::from_string("Error reading file")
                            .with_status_code(StatusCode(500)),
                    );
                    continue;
                }
            };

            if file.seek(SeekFrom::Start(range.start)).is_err() {
                let _ = request.respond(
                    Response::from_string("Range Seek Error").with_status_code(StatusCode(416)),
                );
                continue;
            }

            let mut buffer = vec![0u8; slice_len as usize];
            if file.read_exact(&mut buffer).is_err() {
                let _ = request.respond(
                    Response::from_string("Range Read Error").with_status_code(StatusCode(500)),
                );
                continue;
            }

            let content_range = format!("bytes {}-{}/{}", range.start, range.end, total_size);
            let mut response = Response::from_data(buffer).with_status_code(StatusCode(206));

            response.add_header(Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap());
            response.add_header(
                Header::from_bytes(&b"Content-Range"[..], content_range.as_bytes()).unwrap(),
            );
            response.add_header(Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).unwrap());
            response
                .add_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap());
            if is_cors {
                response.add_header(
                    Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
                );
            }

            let _ = request.respond(response);
            if !is_quiet {
                println!("206 Partial: {} ({})", url, content_range);
            }
        } else {
            // 200 OK
            let mut response = match File::open(&target_path) {
                Ok(file) => Response::from_file(file),
                Err(_) => {
                    let _ = request.respond(
                        Response::from_string("Error opening file")
                            .with_status_code(StatusCode(500)),
                    );
                    continue;
                }
            };

            response.add_header(Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap());
            response.add_header(Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).unwrap());
            response
                .add_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap());
            if is_cors {
                response.add_header(
                    Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
                );
            }

            let _ = request.respond(response);
            if !is_quiet {
                println!("200 OK: {} [{}] ({} bytes)", url, mime, total_size);
            }
        }
    }

    Ok(())
}

fn ctrlc_setup(_running: Arc<AtomicBool>) {
    // Graceful exit handler
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_defaults() {
        let args: Vec<String> = vec![];
        let action = parse_args(args).expect("should parse");
        match action {
            Action::Run(cfg) => {
                assert_eq!(cfg.host, "0.0.0.0");
                assert_eq!(cfg.port, 8080);
                assert!(!cfg.spa);
                assert!(!cfg.cors);
                assert!(!cfg.quiet);
            }
            _ => panic!("Expected Action::Run"),
        }
    }

    #[test]
    fn test_parse_args_flags() {
        let args = vec![
            "-p".to_string(),
            "9090".to_string(),
            "--localhost".to_string(),
            "--spa".to_string(),
            "--cors".to_string(),
            "-q".to_string(),
            "my_site".to_string(),
        ];
        let action = parse_args(args).expect("should parse");
        match action {
            Action::Run(cfg) => {
                assert_eq!(cfg.host, "127.0.0.1");
                assert_eq!(cfg.port, 9090);
                assert!(cfg.spa);
                assert!(cfg.cors);
                assert!(cfg.quiet);
                assert_eq!(cfg.dir, PathBuf::from("my_site"));
            }
            _ => panic!("Expected Action::Run"),
        }
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("hello%20world").unwrap(), "hello world");
        assert_eq!(
            percent_decode("path%2Fwith%2Bplus").unwrap(),
            "path/with+plus"
        );
        assert_eq!(percent_decode("space+plus").unwrap(), "space plus");
    }

    #[test]
    fn test_guess_mime_audio_video() {
        assert_eq!(guess_mime(Path::new("track.mp3")), "audio/mpeg");
        assert_eq!(guess_mime(Path::new("track.opus")), "audio/opus");
        assert_eq!(guess_mime(Path::new("track.ogg")), "audio/ogg");
        assert_eq!(guess_mime(Path::new("video.mp4")), "video/mp4");
        assert_eq!(guess_mime(Path::new("video.webm")), "video/webm");
        assert_eq!(guess_mime(Path::new("app.wasm")), "application/wasm");
    }

    #[test]
    fn test_parse_range_header() {
        let file_size = 1000;
        // bytes=0-499
        let r1 = parse_range_header("bytes=0-499", file_size).unwrap();
        assert_eq!(r1, ByteRange { start: 0, end: 499 });

        // bytes=500-
        let r2 = parse_range_header("bytes=500-", file_size).unwrap();
        assert_eq!(
            r2,
            ByteRange {
                start: 500,
                end: 999
            }
        );

        // bytes=-200 (last 200 bytes)
        let r3 = parse_range_header("bytes=-200", file_size).unwrap();
        assert_eq!(
            r3,
            ByteRange {
                start: 800,
                end: 999
            }
        );

        // Out of bounds
        assert!(parse_range_header("bytes=1500-", file_size).is_none());
    }

    #[test]
    fn test_path_traversal_prevention() {
        let base = Path::new("/tmp/test_dir");
        assert!(resolve_path(base, "/../etc/passwd").is_none());
        assert!(resolve_path(base, "../secret.txt").is_none());
        assert!(resolve_path(base, "foo/../../secret.txt").is_none());
    }
}
