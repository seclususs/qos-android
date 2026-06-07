#!/system/bin/sh

PKG="com.seclususs.qos"

# Force stop package
am force-stop "$PKG" 2>/dev/null

# Uninstall package
pm uninstall "$PKG" 2>/dev/null

# Pause to allow system cleanup
sleep 2

# Remove app installation files
rm -rf /data/app/*/*"$PKG"* 2>/dev/null
rm -rf /data/app/"$PKG"* 2>/dev/null

# Remove main app data & cache
rm -rf /data/data/"$PKG" 2>/dev/null

# Remove Device Encrypted storage data
rm -rf /data/user_de/*/"$PKG" 2>/dev/null

# Remove external storage data
rm -rf /data/media/*/Android/data/"$PKG" 2>/dev/null
