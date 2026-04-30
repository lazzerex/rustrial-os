#!/usr/bin/env python3
"""Combined host-side network test server for QEMU guest validation.

This runs two services on the host:
- HTTP on port 18080
- NTP on port 8123

Guest commands:
- http-get http://10.0.2.2:18080/
- ntp-sync 10.0.2.2:8123

One process is easier than running separate HTTP and NTP helpers.
"""

from __future__ import annotations

import http.server
import socket
import struct
import threading
import time

NTP_UNIX_OFFSET = 2208988800
NTP_PACKET_SIZE = 48


def unix_to_ntp_seconds(unix_seconds: float) -> int:
    return int(unix_seconds) + NTP_UNIX_OFFSET


def build_ntp_response(request: bytes) -> bytes:
    if len(request) < NTP_PACKET_SIZE:
        request = request.ljust(NTP_PACKET_SIZE, b"\x00")

    now = time.time()
    seconds = unix_to_ntp_seconds(now)
    fraction = int((now % 1.0) * (1 << 32)) & 0xFFFFFFFF

    version = (request[0] >> 3) & 0x07
    response = bytearray(NTP_PACKET_SIZE)
    response[0] = ((version & 0x07) << 3) | 4
    response[1] = 1
    response[2] = request[2]
    response[3] = request[3]
    response[12:16] = b"LOCL"
    response[24:32] = request[40:48]
    response[32:40] = struct.pack("!II", seconds, fraction)
    response[40:48] = struct.pack("!II", seconds, fraction)
    return bytes(response)


def run_ntp_server(host: str = "0.0.0.0", port: int = 8123) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((host, port))
    print(f"NTP listening on {host}:{port}")

    while True:
        data, addr = sock.recvfrom(1024)
        sock.sendto(build_ntp_response(data), addr)


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        body = (
            "RustrialOS test server\n"
            f"Path: {self.path}\n"
            f"Time: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}\n"
        ).encode("utf-8")

        self.send_response(200)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format: str, *args) -> None:
        return


def run_http_server(host: str = "0.0.0.0", port: int = 18080) -> None:
    server = http.server.ThreadingHTTPServer((host, port), Handler)
    print(f"HTTP listening on {host}:{port}")
    server.serve_forever()


def main() -> None:
    ntp_thread = threading.Thread(target=run_ntp_server, daemon=True)
    http_thread = threading.Thread(target=run_http_server, daemon=True)

    ntp_thread.start()
    http_thread.start()

    print("Combined network test server running")
    print("  NTP : 10.0.2.2:8123")
    print("  HTTP: 10.0.2.2:18080")

    while True:
        time.sleep(1)


if __name__ == "__main__":
    main()
