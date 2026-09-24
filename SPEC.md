# Mayak Technical Specification (SPEC-01)

> **Document Version:** 1.0.0  
> **Status:** Standard  
> **Standard Body:** Gosplan Ministry of Communications & Radio Broadcasting

---

## 1. Scope & Goals

Mayak is an ultra-lightweight, multi-threaded static HTTP file server written in pure Rust. It delivers static web assets, documentation portals, Single Page Applications, and media streams with zero runtime dependencies.

Key properties:
- Fast startup: under 5 milliseconds.
- High concurrency: Native multi-threaded loop via `tiny_http`.
- Media streaming: RFC 7233 HTTP Range Requests (`206 Partial Content`) for audio/video seeking.
- Resilience: Port auto-increment if the target port is bound by another service.
- Universal binding: `0.0.0.0` by default, with `--localhost` for loopback restriction.

---

## 2. CLI Interface

```
Usage: mk [options] [directory]
       mayak [options] [directory]

Arguments:
  [directory]             Directory to serve (default: current directory or auto-detected site/ / dist/)

Options:
  -p, --port <number>     Initial port to listen on (default: 8080)
  --host <ip>             Host address to bind to (default: 0.0.0.0)
  -l, --localhost         Bind to 127.0.0.1 instead of 0.0.0.0
  -s, --spa               Single Page Application mode: fallback 404 to index.html
  --cors                  Send 'Access-Control-Allow-Origin: *' headers
  -o, --open              Open default browser after starting
  -q, --quiet             Quiet mode (suppress request logs, errors only)
  -h, --help              Print this help information
  -v, --version           Print version information
```

---

## 3. HTTP Protocol Conformance

### 3.1 HTTP Methods
- `GET`: Responds with full file body or range slice.
- `HEAD`: Responds with identical headers as `GET`, without the response body.
- Other methods (`POST`, `PUT`, `DELETE`): Rejected with `405 Method Not Allowed`.

### 3.2 Range Requests (RFC 7233)
If a request contains `Range: bytes=<start>-<end>`:
- The server validates `<start>` and `<end>` against file length.
- If valid, responds with:
  - Status: `206 Partial Content`
  - Header `Content-Range: bytes <start>-<end>/<total>`
  - Header `Accept-Ranges: bytes`
  - Body: requested slice.
- If unsatisfiable, responds with `416 Range Not Satisfiable`.

### 3.3 Security & Directory Traversal
All requested paths are canonicalized against the root served directory:
- Requests containing `..` or leading slashes are validated to ensure they never escape the root.
- Attempts to escape the root directory yield `403 Forbidden` or `404 Not Found`.
- Hidden files/directories (starting with `.`) are ignored or rejected unless explicitly requested.

### 3.4 MIME Types
MIME types are statically determined based on file extension. Unrecognized extensions default to `application/octet-stream`. Text-based formats default to `charset=utf-8`.
