use mayak::{
    execute_bridge_command, find_available_port, guess_mime, parse_args, parse_range_header,
    resolve_path, verify_auth, Action, ByteRange,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[test]
fn test_integration_full_mime_audio_video_and_fonts() {
    // Audio
    assert_eq!(guess_mime(Path::new("song.mp3")), "audio/mpeg");
    assert_eq!(guess_mime(Path::new("song.ogg")), "audio/ogg");
    assert_eq!(guess_mime(Path::new("song.opus")), "audio/opus");
    assert_eq!(guess_mime(Path::new("song.flac")), "audio/flac");
    assert_eq!(guess_mime(Path::new("song.wav")), "audio/wav");
    assert_eq!(guess_mime(Path::new("song.m4a")), "audio/aac");
    assert_eq!(guess_mime(Path::new("song.weba")), "audio/webm");

    // Video
    assert_eq!(guess_mime(Path::new("clip.mp4")), "video/mp4");
    assert_eq!(guess_mime(Path::new("clip.webm")), "video/webm");
    assert_eq!(guess_mime(Path::new("clip.ogv")), "video/ogg");
    assert_eq!(guess_mime(Path::new("clip.mkv")), "video/x-matroska");

    // Web & Fonts
    assert_eq!(guess_mime(Path::new("app.wasm")), "application/wasm");
    assert_eq!(
        guess_mime(Path::new("manifest.webmanifest")),
        "application/manifest+json"
    );
    assert_eq!(guess_mime(Path::new("font.woff2")), "font/woff2");
    assert_eq!(guess_mime(Path::new("image.avif")), "image/avif");
    assert_eq!(
        guess_mime(Path::new("unknown.xyz")),
        "application/octet-stream"
    );
}

#[test]
fn test_integration_range_header_spec() {
    let size = 5000;
    // Standard middle slice
    assert_eq!(
        parse_range_header("bytes=100-199", size),
        Some(ByteRange {
            start: 100,
            end: 199
        })
    );

    // Open-ended slice to EOF
    assert_eq!(
        parse_range_header("bytes=4500-", size),
        Some(ByteRange {
            start: 4500,
            end: 4999
        })
    );

    // Suffix slice (last 500 bytes)
    assert_eq!(
        parse_range_header("bytes=-500", size),
        Some(ByteRange {
            start: 4500,
            end: 4999
        })
    );

    // Beyond EOF
    assert_eq!(parse_range_header("bytes=6000-", size), None);
    assert_eq!(parse_range_header("bytes=5000-6000", size), None);
    assert_eq!(parse_range_header("invalid_header", size), None);
}

#[test]
fn test_integration_safe_path_resolution() {
    let tmp = std::env::temp_dir().join("mayak_test_sandbox");
    let _ = fs::create_dir_all(&tmp);
    let index_file = tmp.join("index.html");
    let mut f = File::create(&index_file).unwrap();
    writeln!(f, "<h1>Gosplan Mayak</h1>").unwrap();

    let sub_dir = tmp.join("sub");
    let _ = fs::create_dir_all(&sub_dir);
    let sub_file = sub_dir.join("data.json");
    let mut f2 = File::create(&sub_file).unwrap();
    writeln!(f2, r#"{{"status":"ok"}}"#).unwrap();

    // Valid resolutions
    assert_eq!(resolve_path(&tmp, "/"), Some(index_file.clone()));
    assert_eq!(resolve_path(&tmp, "/index.html"), Some(index_file.clone()));
    assert_eq!(resolve_path(&tmp, "/sub/data.json"), Some(sub_file));

    // Traversal attacks MUST be blocked
    assert_eq!(resolve_path(&tmp, "/../etc/passwd"), None);
    assert_eq!(resolve_path(&tmp, "/../../root"), None);
    assert_eq!(resolve_path(&tmp, "sub/../../etc/shadow"), None);

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn test_integration_available_port_finder() {
    let port = find_available_port("127.0.0.1", 19000).expect("should find port");
    assert!((19000..=19100).contains(&port));
}

#[test]
fn test_integration_parse_args_all_options() {
    let args = vec![
        "-p".to_string(),
        "3000".to_string(),
        "--host".to_string(),
        "10.0.0.1".to_string(),
        "--spa".to_string(),
        "--cors".to_string(),
        "--open".to_string(),
        "--quiet".to_string(),
        "custom_dir".to_string(),
    ];
    let action = parse_args(args).unwrap();
    match action {
        Action::Run(cfg) => {
            assert_eq!(cfg.port, 3000);
            assert_eq!(cfg.host, "10.0.0.1");
            assert!(cfg.spa);
            assert!(cfg.cors);
            assert!(cfg.open_browser);
            assert!(cfg.quiet);
            assert_eq!(cfg.dir, PathBuf::from("custom_dir"));
            assert_eq!(cfg.bridge, None);
            assert_eq!(cfg.auth, None);
        }
        _ => panic!("Expected Action::Run"),
    }
}

#[test]
fn test_integration_parse_args_bridge_and_auth() {
    let args = vec![
        "--bridge".to_string(),
        "cccp".to_string(),
        "--auth".to_string(),
        "gosplan-secret".to_string(),
    ];
    let action = parse_args(args).unwrap();
    match action {
        Action::Run(cfg) => {
            assert_eq!(cfg.bridge, Some("cccp".to_string()));
            assert_eq!(cfg.auth, Some("gosplan-secret".to_string()));
        }
        _ => panic!("Expected Action::Run"),
    }
}

#[test]
fn test_integration_verify_auth() {
    let token = "komsomol-token-1922";

    // Valid Bearer
    assert!(verify_auth(Some("Bearer komsomol-token-1922"), None, token));
    assert!(verify_auth(Some("bearer komsomol-token-1922"), None, token));

    // Valid X-Mayak-Token
    assert!(verify_auth(None, Some("komsomol-token-1922"), token));

    // Invalid
    assert!(!verify_auth(Some("Bearer wrong-token"), None, token));
    assert!(!verify_auth(None, Some("wrong-token"), token));
    assert!(!verify_auth(Some("Basic 1234"), None, token));
    assert!(!verify_auth(None, None, token));
}

#[test]
fn test_integration_execute_bridge_command() {
    // 1. Echo command
    let res = execute_bridge_command("echo", &["hello".to_string(), "world".to_string()], None)
        .expect("echo should execute");
    assert!(res.ok);
    assert_eq!(res.exit_code, 0);
    assert_eq!(res.stdout.trim(), "hello world");

    // 2. Command with stdin
    let res_cat = execute_bridge_command("cat", &[], Some("Gosplan Transmitter"))
        .expect("cat should execute");
    assert!(res_cat.ok);
    assert_eq!(res_cat.exit_code, 0);
    assert_eq!(res_cat.stdout, "Gosplan Transmitter");

    // 3. Failing command
    let res_false = execute_bridge_command("false", &[], None).expect("false should execute");
    assert!(!res_false.ok);
    assert_ne!(res_false.exit_code, 0);

    // 4. Non-existent binary
    assert!(execute_bridge_command("non_existent_binary_xyz_123", &[], None).is_err());
}
