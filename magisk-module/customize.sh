#!/system/bin/sh

set_perm_recursive "$MODPATH" 0 0 0755 0644

ui_print "[*] Extracting..."
unzip -o "$ZIPFILE" 'service.sh' 'system/bin/adaptive_daemon' -d "$MODPATH" >&2

ui_print "[*] Setting permissions..."
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system/bin/adaptive_daemon" 0 0 0755

[ -f "$MODPATH/common/install.sh" ] && . "$MODPATH"/common/install.sh
[ -d "$MODPATH/common" ] && rm -rf "$MODPATH"/common/*.sh

for file in $(find "$MODPATH" -type f -name "*.sh" -o -name "*.prop" -o -name "*.rule"); do
  [ -f "$file" ] && {
    sed -i -e "/^[[:blank:]]*#/d" -e "/^ *$/d" "$file"
    [ "$(tail -1 "$file")" ] && echo "" >>"$file"
  }
done

find "$MODPATH" -empty -type d -delete
[ -e /data/system/package_cache ] && rm -rf /data/system/package_cache/*

ui_print "Successfully..."