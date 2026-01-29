#!/system/bin/sh
SKIPUNZIP=1

MODID="sys_qos"
ACTIVE_DIR="/data/adb/modules/$MODID"

grep_prop() {
  local REGEX="s/^$1=//p"
  sed -n "$REGEX" "$2"
}

get_prop() {
  local prop=$(getprop "$1")
  echo "$prop"
}

unzip -o "$ZIPFILE" 'module.prop' -d "$MODPATH" >&2

ui_print_header() {
  ui_print " "
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠤⣲⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠖⠋⢀⠞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠜⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠊⠀⠀⡠⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠊⡰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⠁⠀⠀⡜⠁⠀⠀⠀⠀⠀⠀⠀⠀⣀⠤⠚⠁⡜⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⢸⠀⠀⠀⠀⢀⣀⣀⠤⠖⠈⠀⠀⢀⡜⠀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠓⠲⢤⣸⠒⣊⣭⠛⠉⠀⠀⠀⠀⠀⢀⣠⢿⡶⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠇⠀⠀⠀⠀⣹⠎⠀⠀⠑⡄⠀⢀⡠⠔⢊⡥⢺⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⠎⠀⠀⠀⣠⠞⠁⠀⠀⠀⢀⣾⠋⠁⣠⠞⠁⠀⢸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠃⠀⡠⠊⡜⠁⠀⠀⠀⢀⡊⠁⠁⠀⢊⡀⠀⠀⠀⣀⣉⣓⣦⡤⠤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⡤⠊⠁⠸⠀⠀⠀⡠⡖⡝⠀⠀⠀⠀⠀⠈⢉⡩⠭⠒⢋⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠁⠀⠀⠀⠑⠒⠛⠒⠋⠁⠀⠀⠀⠀⠀⠀⠘⠤⣀⡀⠈⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠜⠁⠀⠀⠀⠀⠀⠀⢀⣀⠤⠄⠀⠀⠀⡰⠚⢧⠉⠒⠒⠮⠽⣾⣦⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠋⠁⡠⣖⠂⠀⠀⠀⡠⠋⠉⠀⡀⠀⠀⢀⡴⠁⠀⠸⡄⠀⠀⠀⠀⡇⠙⢌⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠘⠐⠁⣀⡠⠔⠋⣀⣀⡴⠚⠓⡶⣞⣉⣀⣀⡠⢤⠇⠀⠀⠀⢰⣃⡀⠈⢳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢧⣀⣠⡊⠁⡀⣠⠞⠁⠀⠀⠀⡜⠁⠀⠀⠀⠀⠀⡜⠀⠀⠀⠀⣿⠀⠈⠑⢄⢳⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠰⣽⢻⡏⠁⠀⠀⠀⢀⠞⠑⠦⠤⠤⠤⠄⡸⠁⠀⠀⠀⢸⠉⣆⠀⠀⠘⡾⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⠀⠃⠀⠀⠀⢀⢏⠀⠀⠀⠀⠀⠀⡰⠁⠀⠀⠀⠀⢸⠀⠘⡄⠀⠀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
  ui_print "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠑⠦⠤⠤⠄⢲⠁⠀⠀⠀⠀⠀⠘⣆⣀⣹"
  ui_print " "
  ui_print "  QoS"
  ui_print "  Version : v$(grep_prop version "$MODPATH/module.prop")"
  ui_print "-----------------------------------"
}

ui_print_log() { ui_print "  ● $1"; }
ui_print_info() { ui_print "    ➜ $1"; }
ui_print_warn() { ui_print "    ! $1"; }
ui_print_err() { ui_print "    X $1"; }

# Original concept by Chainfire, modernized for new Android versions
chooseport() {
  sleep 0.5
  timeout 0.2 /system/bin/getevent -l -c 1 > /dev/null 2>&1
  while true; do
    timeout 15 /system/bin/getevent -l -c 1 > "$TMPDIR/events" 2>/dev/null

    if [ -s "$TMPDIR/events" ]; then
      if grep -qE "KEY_VOLUMEUP| 0073 " "$TMPDIR/events"; then
        return 0
      fi
      if grep -qE "KEY_VOLUMEDOWN| 0072 " "$TMPDIR/events"; then
        return 1
      fi
    fi

  done
}

backup_config() {
  if [ -f "$ACTIVE_DIR/config.ini" ]; then
    ui_print_log "Detecting previous installation..."
    cp -f "$ACTIVE_DIR/config.ini" "$TMPDIR/config.ini.bak"
    ui_print_info "Old config backed up."
    return 0
  fi
  return 1
}

restore_config() {
  local OLD_CFG="$TMPDIR/config.ini.bak"
  local NEW_CFG="$MODPATH/config.ini"

  if [ -f "$OLD_CFG" ]; then
    ui_print_log "Restoring user configuration..."
    
    grep '^[a-zA-Z0-9_.]\+=' "$OLD_CFG" | while read -r line; do
      local key=$(echo "$line" | cut -d'=' -f1)
      local val=$(echo "$line" | cut -d'=' -f2-)
      if grep -q "^$key=" "$NEW_CFG"; then
        sed -i "s|^$key=.*|$key=$val|" "$NEW_CFG"
      fi
    done

    ui_print_info "Config merged. User settings restored."
  else
    ui_print_log "Using default configuration."
  fi
}

run_setup_wizard() {
  ui_print_log "Starting Setup..."
  ui_print " "
  
  FEATURES="Cleaner:cleaner_enabled CPU_Controller:cpu_enabled Storage_Controller:storage_enabled Display_Tweaks:display_enabled System_Tweaks:tweaks_enabled"
  
  for item in $FEATURES; do
    local name=$(echo "$item" | cut -d':' -f1 | tr '_' ' ')
    local key=$(echo "$item" | cut -d':' -f2)
    local warning=""
    
    case "$key" in
      "cpu_enabled")
         [ ! -e "/proc/pressure/cpu" ] && warning="PSI CPU missing (/proc/pressure/cpu). Service might degrade."
         ;;
      "storage_enabled")
         [ ! -e "/proc/pressure/io" ] && warning="PSI IO missing (/proc/pressure/io). Service will fail."
         ;;
      "display_enabled")
         local DEV=$(get_prop "ro.product.device")
         local BID=$(get_prop "ro.build.id")
         if [ "$DEV" != "selene" ] || [ "$BID" != "TQ3A.230901.001.B1" ]; then
            warning="Device mismatch ($DEV). Feature might be auto-disabled."
         fi
         ;;
      "cleaner_enabled")
         if [ ! -d "/data/data" ] || [ ! -d "/proc" ]; then
            warning="System paths inaccessible (/data/data or /proc)"
         elif [ ! -e "/proc/pressure/cpu" ] || [ ! -e "/proc/pressure/io" ]; then
            warning="PSI metrics missing. Cleaner safety checks will fail."
         fi
         ;;
    esac

    ui_print "  [?] Enable $name?"

    if [ ! -z "$warning" ]; then
       ui_print_warn "$warning"
    fi

    ui_print "    (+) Vol Up   = ENABLE"
    ui_print "    (-) Vol Down = DISABLE"
    
    if chooseport; then
      sed -i "s|^$key=.*|$key=true|" "$MODPATH/config.ini"
      ui_print_info "$name -> ON"
    else
      sed -i "s|^$key=.*|$key=false|" "$MODPATH/config.ini"
      ui_print_warn "$name -> OFF"
    fi
    ui_print " "
    sleep 0.2
  done

  ui_print_info "Configuration Setup Complete."
}

ask_bootanimation() {
  ui_print " "
  ui_print "  [?] Install Custom Bootanimation?"
  ui_print "    (+) Vol Up   : YES (Install)"
  ui_print "    (-) Vol Down : NO  (Skip)"
  
  if chooseport; then
    ui_print_info "Bootanimation: ENABLED"
    return 0
  else
    ui_print_warn "Bootanimation: SKIPPED"
    return 1
  fi
}

REQUIRED_PROPS="
ro.vendor.mtk.bt_sap_enable
ro.vendor.mtk_wappush_support
ro.vendor.mtk_c2k_support
ro.vendor.mtk_c2k_lte_mode
ro.vendor.mtk_embms_support
ro.vendor.mtk_md_world_mode_support
ro.vendor.connsys.dedicated.log
ro.vendor.mtk_protocol1_rat_config
ro.vendor.mtk_wapi_support
"

validate_system_props() {
  local remove_system_prop=0

  for prop in $REQUIRED_PROPS; do
    key=$(echo "$prop" | cut -d'=' -f1)
    value=$(echo "$prop" | cut -d'=' -f2-)

    actual_value=$(get_prop "$key")
    if [ -z "$actual_value" ]; then
      ui_print_warn "Missing prop: $key"
      remove_system_prop=1
    fi
  done

  if [ $remove_system_prop -eq 1 ]; then
    if [ -f "$MODPATH/system.prop" ]; then
      ui_print_warn "Some required props missing, removing system.prop"
      rm -f "$MODPATH/system.prop"
    fi
  fi
}

ui_print_header

if [ "$ARCH" != "arm64" ]; then
  ui_print_warn "Architecture $ARCH might be incompatible (Daemon targets arm64)."
fi

backup_config
HAS_BACKUP=$?

ui_print_log "Extracting module files..."
unzip -o "$ZIPFILE" 'service.sh' 'system/bin/qos_daemon' 'config.ini' 'system.prop' 'system/product/media/bootanimation.zip' -d "$MODPATH" >&2
unzip -o "$ZIPFILE" 'common/*' -d "$MODPATH" >&2

validate_system_props

ui_print " "
if [ $HAS_BACKUP -eq 0 ]; then
  ui_print_warn "Previous Configuration Found!"
  ui_print "    (+) Vol Up   : MERGE (Keep settings)"
  ui_print "    (-) Vol Down : RESET (Re-configure)"
  
  if chooseport; then
    restore_config
  else
    ui_print_log "Old config discarded."
    run_setup_wizard
  fi
else
  ui_print_log "Fresh Installation Detected."
  ui_print "    (+) Vol Up   : CUSTOMIZE Features"
  ui_print "    (-) Vol Down : USE DEFAULTS"
  
  if chooseport; then
    run_setup_wizard
  else
    ui_print_info "Using default configuration."
  fi
fi

INSTALL_BOOTANIM=0
if ask_bootanimation; then
  INSTALL_BOOTANIM=1
fi

ui_print_log "Setting permissions..."
set_perm_recursive "$MODPATH" 0 0 0755 0644
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system/bin/qos_daemon" 0 0 0755
set_perm "$MODPATH/config.ini" 0 0 0644

if [ -d "$MODPATH/common" ]; then
  ui_print_log "Running additional scripts..."
  
  if [ $INSTALL_BOOTANIM -eq 1 ]; then
    export BootAnimation_location='/system/product/media/bootanimation.zip'
  else
    rm -f "$MODPATH/system/product/media/bootanimation.zip" 2>/dev/null
    rmdir -p "$MODPATH/system/product" 2>/dev/null
    unset BootAnimation_location
  fi

  for script in "$MODPATH"/common/*.sh; do
    if [ -f "$script" ]; then
        . "$script"
    fi
  done
  rm -rf "$MODPATH/common"
fi

ui_print_log "Cleaning up..."
rm -f "$MODPATH/customize.sh" 2>/dev/null
find "$MODPATH" -empty -type d -delete
[ -e /data/system/package_cache ] && rm -rf /data/system/package_cache/*