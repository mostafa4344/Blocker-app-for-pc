#!/bin/sh
# FocusFort installer for macOS (launchd daemon). Run with sudo from the release folder.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
install -m 0755 "$here/focusfort-agent" /usr/local/bin/focusfort-agent
install -m 0755 "$here/uninstall.sh" /usr/local/bin/focusfort-uninstall
install -m 0644 "$here/app.focusfort.agent.plist" /Library/LaunchDaemons/app.focusfort.agent.plist
launchctl bootout system/app.focusfort.agent 2>/dev/null || true
launchctl bootstrap system /Library/LaunchDaemons/app.focusfort.agent.plist
echo "FocusFort installed. Open http://127.0.0.1:17345/"
