#!/bin/sh
# ContentGuard uninstaller for macOS. Honours uninstall protection.
set -eu
if ! /usr/local/bin/contentguard-agent uninstall-check; then
  echo "Open ContentGuard > Protection > Request uninstall, then run this again once approved." >&2
  exit 2
fi
launchctl bootout system/app.contentguard.agent 2>/dev/null || true
/usr/local/bin/contentguard-agent cleanup || true
rm -f /Library/LaunchDaemons/app.contentguard.agent.plist /usr/local/bin/contentguard-agent /usr/local/bin/contentguard-uninstall
echo "ContentGuard removed (settings kept in /Library/Application Support/ContentGuard)."
