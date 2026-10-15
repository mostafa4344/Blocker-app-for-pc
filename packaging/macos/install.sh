#!/bin/sh
# ContentGuard installer for macOS (launchd daemon). Run with sudo from the release folder.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
install -m 0755 "$here/contentguard-agent" /usr/local/bin/contentguard-agent
install -m 0755 "$here/uninstall.sh" /usr/local/bin/contentguard-uninstall
install -m 0644 "$here/app.contentguard.agent.plist" /Library/LaunchDaemons/app.contentguard.agent.plist
launchctl bootout system/app.contentguard.agent 2>/dev/null || true
launchctl bootstrap system /Library/LaunchDaemons/app.contentguard.agent.plist
echo "ContentGuard installed. Open http://127.0.0.1:17345/"
