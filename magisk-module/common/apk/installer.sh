#!/system/bin/sh

while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

MOD_DIR="/data/adb/modules/sys_qos"
APK_DIR="$MOD_DIR/common/apk"
APK_PATH="$APK_DIR/com/seclususs/qos/com.seclususs.qos.apk"
PKG="com.seclususs.qos"

if [ -f "$APK_PATH" ]; then
    pm install -r -d -g "$APK_PATH"
fi

if [ -d "$APK_DIR" ]; then
    rm -rf "$APK_DIR"
fi

rm -f "$0"
