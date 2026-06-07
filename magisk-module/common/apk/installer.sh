#!/system/bin/sh

while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

MOD_DIR="/data/adb/modules/sys_qos"
COMMON_DIR="$MOD_DIR/common"
APK_PATH="$COMMON_DIR/apk/com/seclususs/qos/com.seclususs.qos.apk"
PKG="com.seclususs.qos"

if [ -f "$APK_PATH" ]; then
    pm install -r -d -g "$APK_PATH"
fi

if [ -d "$COMMON_DIR" ]; then
    rm -rf "$COMMON_DIR"
fi

rm -f "$0"
