#!/bin/sh
# FocusFort uninstaller for macOS. Honours uninstall protection.
set -eu
if ! /usr/local/bin/focusfort-agent uninstall-check; then
  echo "Open FocusFort > Protection > Request uninstall, then run this again once approved." >&2
  exit 2
fi
launchctl bootout system/app.focusfort.agent 2>/dev/null || true
/usr/local/bin/focusfort-agent cleanup || true
rm -f /Library/LaunchDaemons/app.focusfort.agent.plist /usr/local/bin/focusfort-agent /usr/local/bin/focusfort-uninstall
echo "FocusFort removed (settings kept in /Library/Application Support/FocusFort)."
