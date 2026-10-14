#!/bin/sh
# FocusFort installer for Linux (systemd). Run with sudo from the release folder.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
install -m 0755 "$here/focusfort-agent" /usr/local/bin/focusfort-agent
install -m 0644 "$here/focusfort.service" /etc/systemd/system/focusfort.service
install -m 0755 "$here/uninstall.sh" /usr/local/bin/focusfort-uninstall
[ -d "$here/extension" ] && { mkdir -p /usr/local/share/focusfort; cp -r "$here/extension" /usr/local/share/focusfort/; }
systemctl daemon-reload
systemctl enable --now focusfort.service
echo "FocusFort installed. Open http://127.0.0.1:17345/"
