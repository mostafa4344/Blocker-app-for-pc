#!/bin/sh
# FocusFort uninstaller for Linux. Honours uninstall protection.
set -eu
if ! /usr/local/bin/focusfort-agent uninstall-check; then
  echo "Open FocusFort > Protection > Request uninstall, then run this again once approved." >&2
  exit 2
fi
systemctl disable --now focusfort.service || true
/usr/local/bin/focusfort-agent cleanup || true
rm -f /etc/systemd/system/focusfort.service /usr/local/bin/focusfort-agent /usr/local/bin/focusfort-uninstall
rm -rf /usr/local/share/focusfort
systemctl daemon-reload
echo "FocusFort removed (settings kept in /var/lib/focusfort)."
