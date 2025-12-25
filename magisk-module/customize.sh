#!/system/bin/sh

ui_print_header() {
ui_print " "
  ui_print "⠀⠀⠀⠀⠀⠀⢀⣴⣾⣿⣿⣿⣿⣿⣿⣷⣦⡀⠀⠀⠀⠀⠐⣶⣄⠀⠀"
  ui_print "⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⡀⠀⠀⠀⢹⣿⣆⠀"
  ui_print "⠀⠀⠀⢠⣿⣿⣿⡿⠋⠉⠻⣿⣿⠟⠉⠙⢿⣿⣿⣿⡄⠀⠀⢸⣿⣿⠀"
  ui_print "⠀⠀⠀⣾⣿⣿⣿⠁⠀⣠⣤⡈⠁⢠⣄⡀⠈⣿⣿⣿⣷⠀⠀⣾⣿⡏⠀"
  ui_print "⠀⠀⢸⣿⣿⣿⣿⠀⢰⣿⣿⡇⠀⢸⣿⣿⡆⢸⣿⣿⣿⣿⣿⣿⡟⠀⠀"
  ui_print "⠀⠀⠈⣿⣿⣿⣿⡆⠈⠛⠛⠁⠀⠈⠛⠛⠁⣼⣿⣿⣿⣿⣿⡿⠁⠀⠀"
  ui_print "⠀⠀⠀⠹⣿⣿⣿⣿⣶⣤⣤⣄⣀⣠⣤⣤⣾⣿⣿⣿⣿⣿⠟⠁⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠈⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠁⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠈⠉⠛⠛⠿⠿⠿⠿⠛⠛⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print " "
  ui_print "  unsafe { // Trust me, bro"
  ui_print "      cpp_bridge::trigger_segfault();"
  ui_print "  }"
  ui_print " "
  ui_print "  QoS v$(grep_prop version "$MODPATH/module.prop")"
  ui_print "  [ MEMORY_SAFETY: OPTIONAL ]"
  ui_print "  --------------------------------------------------"
  ui_print " "
}

ui_print_log() {
  ui_print "  ● $1"
}

ui_print_info() {
  ui_print "    ➜ $1"
}

grep_prop() {
  local REGEX="s/^$1=//p"
  sed -n "$REGEX" "$2"
}

set_perm_recursive "$MODPATH" 0 0 0755 0644

ui_print_header

ui_print_log "Extracting module files..."
unzip -o "$ZIPFILE" 'service.sh' 'system/bin/qos_daemon' 'config.ini' -d "$MODPATH" >&2
unzip -o "$ZIPFILE" 'common/*' -d "$MODPATH" >&2

ui_print_log "Checking configuration..."

if [ -f "/data/adb/modules/sys_qos/config.ini" ]; then
  ui_print_info "Preserving existing config.ini"
  cp -f "/data/adb/modules/sys_qos/config.ini" "$MODPATH/config.ini"
else
  ui_print_info "Using default config.ini"
fi

ui_print_log "Setting permissions..."
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system/bin/qos_daemon" 0 0 0755
set_perm "$MODPATH/config.ini" 0 0 0644

if [ -d "$MODPATH/common" ]; then
  ui_print_log "Running additional scripts..."
  export BootAnimation_location='/system/product/media/bootanimation.zip'
  for script in "$MODPATH"/common/*.sh; do
    if [ -f "$script" ]; then
        . "$script"
    fi
  done
  rm -rf "$MODPATH/common"
fi

ui_print_log "Cleaning up..."

for file in $(find "$MODPATH" -type f -name "*.sh" -o -name "*.prop" -o -name "*.rule"); do
  if [ -f "$file" ]; then
    sed -i -e "/^[[:blank:]]*#/d" -e "/^ *$/d" "$file"
    [ "$(tail -1 "$file")" ] && echo "" >>"$file"
  fi
done

find "$MODPATH" -empty -type d -delete
[ -e /data/system/package_cache ] && rm -rf /data/system/package_cache/*