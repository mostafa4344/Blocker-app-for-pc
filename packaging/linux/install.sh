#!/bin/sh
# ContentGuard installer for Linux (systemd). Run with sudo from the release folder.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
install -m 0755 "$here/contentguard-agent" /usr/local/bin/contentguard-agent
install -m 0644 "$here/contentguard.service" /etc/systemd/system/contentguard.service
install -m 0755 "$here/uninstall.sh" /usr/local/bin/contentguard-uninstall
[ -d "$here/extension" ] && { mkdir -p /usr/local/share/contentguard; cp -r "$here/extension" /usr/local/share/contentguard/; }
systemctl daemon-reload
systemctl enable --now contentguard.service
echo "ContentGuard installed. Open http://127.0.0.1:17345/"
