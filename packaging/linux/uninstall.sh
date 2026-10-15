#!/bin/sh
# ContentGuard uninstaller for Linux. Honours uninstall protection.
set -eu
if ! /usr/local/bin/contentguard-agent uninstall-check; then
  echo "Open ContentGuard > Protection > Request uninstall, then run this again once approved." >&2
  exit 2
fi
systemctl disable --now contentguard.service || true
/usr/local/bin/contentguard-agent cleanup || true
rm -f /etc/systemd/system/contentguard.service /usr/local/bin/contentguard-agent /usr/local/bin/contentguard-uninstall
rm -rf /usr/local/share/contentguard
systemctl daemon-reload
echo "ContentGuard removed (settings kept in /var/lib/contentguard)."
