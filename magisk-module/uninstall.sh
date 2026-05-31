#!/system/bin/sh

PKG="com.seclususs.qos"

# Uninstall package
pm uninstall "$PKG"

# Remove external storage data (Android/data)
rm -rf /data/media/0/Android/data/$PKG
