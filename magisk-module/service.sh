#!/system/bin/sh

MODDIR="${0%/*}"
DAEMON_BIN="$MODDIR/system/bin/qos_daemon"
DAEMON_NAME="qos_daemon"

while [ "$(getprop sys.boot_completed)" != "1" ]; do
  sleep 1
done

if pgrep -f "$DAEMON_NAME" > /dev/null; then
  exit 0
fi

$DAEMON_BIN &