#!/usr/bin/env python3

import argparse
import http.server
import socketserver

DEFAULT_PORT = 8081


def main():
    parser = argparse.ArgumentParser(
        description='Debug HTTP server supporting WASM')
    parser.add_argument('-p',
                        '--port',
                        default=DEFAULT_PORT,
                        help='port to be served')
    args = parser.parse_args()

    handler = http.server.SimpleHTTPRequestHandler
    handler.extensions_map[".wasm"] = "application/wasm"

    with socketserver.TCPServer(("", args.port), handler) as httpd:
        print(
            f"info: serving HTTP on port {args.port} (http://localhost:{args.port})"
        )
        httpd.serve_forever()


if __name__ == "__main__":
    main()
