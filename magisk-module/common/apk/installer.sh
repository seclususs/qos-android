#!/system/bin/sh

while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

APK_PATH="/data/adb/modules/sys_qos/common/apk/com/seclususs/qos/com.seclususs.qos.apk"
PKG="com.seclususs.qos"

if [ -f "$APK_PATH" ]; then
    if ! pm list packages | grep -q "$PKG"; then
        pm install -g "$APK_PATH"
    fi
fi

rm -f "$0"
