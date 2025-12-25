#!/system/bin/sh

FILE_CHECK="$MODPATH/system/product/media/bootanimation.zip"

if [ -f "$FILE_CHECK" ] || [ -f "$BootAnimation_location" ]; then
    ui_print "    ➜ [+] Boot-Animation Installed"
else
    ui_print "    ➜ [!] No Boot-Animation found (Skipping)"
fi