#!/system/bin/sh

PKG="com.seclususs.qos"

# Remove app installation files (APK & splits)
rm -rf /data/app/*/*$PKG*
rm -rf /data/app/$PKG*

# Remove main app data & cache
rm -rf /data/data/$PKG

# Remove Device Encrypted (DE) storage data
rm -rf /data/user_de/0/$PKG

# Remove external storage data (Android/data)
rm -rf /data/media/0/Android/data/$PKG
