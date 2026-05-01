//! HTTP (Hypertext Transfer Protocol) Client Implementation
//!
//! RFC 7230: https://tools.ietf.org/html/rfc7230
//!
//! Basic HTTP client for fetching web resources. Builds on TCP for reliable
//! connection and implements core HTTP/1.1 features (GET, Host header, etc).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::net::Ipv4Addr;

use crate::net::tcp::{tcp_connect, tcp_send, tcp_recv, tcp_close};
use crate::net::tcp::{get_connection_state, TcpState};
use crate::task::yield_now;

/// HTTP default port
pub const HTTP_PORT: u16 = 80;

/// HTTP error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpError {
    /// TCP connection failed
    ConnectionFailed,
    /// Failed to send HTTP request
    SendFailed,
    /// Failed to receive HTTP response
    RecvFailed,
    /// Timeout waiting for response
    Timeout,
    /// Invalid HTTP response
    InvalidResponse,
    /// 4xx or 5xx status code
    HttpError,
    /// DNS resolution failed
    DnsError,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpError::ConnectionFailed => write!(f, "HTTP connection failed"),
            HttpError::SendFailed => write!(f, "Failed to send HTTP request"),
            HttpError::RecvFailed => write!(f, "Failed to receive HTTP response"),
            HttpError::Timeout => write!(f, "HTTP timeout"),
            HttpError::InvalidResponse => write!(f, "Invalid HTTP response"),
            HttpError::HttpError => write!(f, "HTTP error (4xx/5xx)"),
            HttpError::DnsError => write!(f, "DNS resolution failed"),
        }
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code (200, 404, 500, etc)
    pub status_code: u16,
    /// Response headers
    pub headers: Vec<(String, String)>,
    /// Response body
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Get header value
    pub fn header(&self, name: &str) -> Option<&str> {
        for (key, value) in &self.headers {
            if key.eq_ignore_ascii_case(name) {
                return Some(value);
            }
        }
        None
    }

    /// Get content length
    pub fn content_length(&self) -> Option<usize> {
        self.header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
}

/// Parse HTTP response from bytes
fn parse_response(data: &[u8]) -> Result<HttpResponse, HttpError> {
    // Split headers and body
    let response_str = core::str::from_utf8(data).map_err(|_| HttpError::InvalidResponse)?;
    
    let (headers_str, body_str) = if let Some(pos) = response_str.find("\r\n\r\n") {
        (&response_str[..pos], &response_str[pos + 4..])
    } else if let Some(pos) = response_str.find("\n\n") {
        (&response_str[..pos], &response_str[pos + 2..])
    } else {
        return Err(HttpError::InvalidResponse);
    };

    // Parse status line
    let mut lines = headers_str.lines();
    let status_line = lines.next().ok_or(HttpError::InvalidResponse)?;
    
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(HttpError::InvalidResponse);
    }

    let status_code = parts[1].parse::<u16>().map_err(|_| HttpError::InvalidResponse)?;

    // Parse headers
    let mut headers = Vec::new();
    for line in lines {
        if let Some(colon_pos) = line.find(':') {
            let key = String::from(line[..colon_pos].trim());
            let value = String::from(line[colon_pos + 1..].trim());
            headers.push((key, value));
        }
    }

    Ok(HttpResponse {
        status_code,
        headers,
        body: body_str.as_bytes().to_vec(),
    })
}

/// Perform HTTP GET request
///
/// # Arguments
/// * `url` - URL to fetch (http://host/path)
/// * `local_ip` - Local IP address for socket
///
/// # Returns
/// * `Ok(HttpResponse)` - HTTP response
/// * `Err(HttpError)` - Request failed
pub async fn http_get(url: &str, local_ip: Ipv4Addr) -> Result<HttpResponse, HttpError> {
    crate::serial_println!("[HTTP] GET {}", url);

    // Parse URL (simplified: http://host/path)
    let url = if url.starts_with("http://") {
        &url[7..]
    } else {
        url
    };

    let (host, path) = if let Some(slash_pos) = url.find('/') {
        (&url[..slash_pos], &url[slash_pos..])
    } else {
        (url, "/")
    };

    // Parse host and port
    let (host_addr, port) = if let Some(colon_pos) = host.find(':') {
        let h = &host[..colon_pos];
        let p = host[colon_pos + 1..]
            .parse::<u16>()
            .unwrap_or(HTTP_PORT);
        (h, p)
    } else {
        (host, HTTP_PORT)
    };

    // Resolve hostname to IP (simple for now)
    let remote_ip = host_addr.parse::<Ipv4Addr>()
        .map_err(|_| HttpError::DnsError)?;

    crate::serial_println!("[HTTP] Connecting to {}:{}", remote_ip, port);

    // Connect via TCP
    let socket_id = tcp_connect(remote_ip, port, local_ip)
        .map_err(|_| HttpError::ConnectionFailed)?;

    // Wait for the 3-way handshake to complete before sending application data.
    let mut connected = false;
    for _ in 0..1000 {
        if matches!(get_connection_state(socket_id), Some(TcpState::Established)) {
            connected = true;
            break;
        }
        yield_now().await;
    }

    if !connected {
        let _ = tcp_close(socket_id);
        return Err(HttpError::Timeout);
    }

    // Build HTTP request
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );

    crate::serial_println!("[HTTP] Sending request ({} bytes)", request.len());

    // Send request
    tcp_send(socket_id, request.as_bytes())
        .map_err(|_| HttpError::SendFailed)?;

    // Receive response (simplified: expect < 4KB)
    let mut response_data = Vec::new();
    const TIMEOUT_ITERATIONS: u32 = 1000;
    
    for _ in 0..TIMEOUT_ITERATIONS {
        if let Some(state) = get_connection_state(socket_id) {
            match state {
                TcpState::Closed | TcpState::TimeWait => {
                    crate::serial_println!("[HTTP] Connection closed by peer");
                    break;
                }
                _ => {}
            }
        }

        match tcp_recv(socket_id, 1024) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    break;
                }
                response_data.extend_from_slice(&chunk);
            }
            Err(_) => {
                crate::task::yield_now().await;
            }
        }
    }

    // Close connection
    let _ = tcp_close(socket_id);

    crate::serial_println!("[HTTP] Received {} bytes", response_data.len());

    // Parse response
    parse_response(&response_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_response() {
        let response_data = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello";
        let response = parse_response(response_data).unwrap();
        
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"Hello");
        assert_eq!(response.content_length(), Some(5));
        assert!(response.is_success());
    }

    #[test]
    fn test_parse_404_response() {
        let response_data = b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        let response = parse_response(response_data).unwrap();
        
        assert_eq!(response.status_code, 404);
        assert!(!response.is_success());
    }

    #[test]
    fn test_header_lookup() {
        let response = HttpResponse {
            status_code: 200,
            headers: vec![
                (String::from("Content-Type"), String::from("text/plain")),
                (String::from("Content-Length"), String::from("42")),
            ],
            body: Vec::new(),
        };

        assert_eq!(response.header("Content-Type"), Some("text/plain"));
        assert_eq!(response.header("content-type"), Some("text/plain"));
        assert_eq!(response.header("Missing"), None);
    }
}
