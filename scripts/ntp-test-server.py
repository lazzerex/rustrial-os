#!/usr/bin/env python3
"""Tiny NTP responder for local QEMU testing.

Bind this on the host and point the guest at 10.0.2.2:8123:

    python3 scripts/ntp-test-server.py --port 8123

Then inside Rustrial OS:

    ntp-sync 10.0.2.2:8123

This returns a minimal server-mode NTP reply using the host's current UTC time.
"""

from __future__ import annotations

import argparse
import socket
import struct
import time

NTP_UNIX_OFFSET = 2208988800
PACKET_SIZE = 48


def unix_to_ntp_seconds(unix_seconds: float) -> int:
    return int(unix_seconds) + NTP_UNIX_OFFSET


def build_response(request: bytes) -> bytes:
    if len(request) < PACKET_SIZE:
        request = request.ljust(PACKET_SIZE, b"\x00")

    now = time.time()
    ntp_seconds = unix_to_ntp_seconds(now)
    fraction = int((now % 1.0) * (1 << 32)) & 0xFFFFFFFF

    leap = 0
    version = (request[0] >> 3) & 0x07
    mode = 4

    response = bytearray(PACKET_SIZE)
    response[0] = (leap << 6) | ((version & 0x07) << 3) | mode
    response[1] = 1  # stratum
    response[2] = request[2]
    response[3] = request[3]
    response[4:8] = struct.pack("!I", 0)
    response[8:12] = struct.pack("!I", 0)
    response[12:16] = b"LOCL"
    response[16:24] = request[40:48]  # reference timestamp not important here
    response[24:32] = request[40:48]  # originate = client's transmit timestamp
    response[32:40] = struct.pack("!II", ntp_seconds, fraction)
    response[40:48] = struct.pack("!II", ntp_seconds, fraction)
    return bytes(response)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8123)
    args = parser.parse_args()

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((args.host, args.port))
    print(f"NTP test server listening on {args.host}:{args.port}")

    while True:
        data, addr = sock.recvfrom(1024)
        response = build_response(data)
        sock.sendto(response, addr)


if __name__ == "__main__":
    main()