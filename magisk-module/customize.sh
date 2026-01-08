#!/system/bin/sh

MODID="sys_qos"
ACTIVE_DIR="/data/adb/modules/$MODID"

grep_prop() {
  local REGEX="s/^$1=//p"
  sed -n "$REGEX" "$2"
}

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

ui_print_log() { ui_print "  ● $1"; }
ui_print_info() { ui_print "    ➜ $1"; }
ui_print_warn() { ui_print "    ! $1"; }

keytest() {
  ui_print "    - Press Vol Up (+) to CONTINUE (Risk)"
  ui_print "    - Press Vol Down (-) to CANCEL"
  (/system/bin/getevent -lc 1 2>&1 | /system/bin/grep DELAY | /system/bin/awk '{ print $3 }') &
  while true; do
    sleep 0.1
    if [ -z "$(pidof getevent)" ]; then
        return 1
    fi
  done
}

chooseport() {
  # Original idea by chainfire @xda-developers, improved by various magisk devs
  while true; do
    /system/bin/getevent -lc 1 2>&1 | /system/bin/grep DELAY | /system/bin/awk '{ print $3 }' > $TMPDIR/events
    if [ -s $TMPDIR/events ]; then
        case $(cat $TMPDIR/events) in
        "0073"|"KEY_VOLUMEUP")
            return 0
            ;;
        "0072"|"KEY_VOLUMEDOWN")
            return 1
            ;;
        esac
    fi
    rm -f $TMPDIR/events
    sleep 0.2
  done
}

check_strict_paths() {
  ui_print_log "Validating system paths..."
  
  MISSING_LIST=""
  MISSING_COUNT=0

  PATHS_PSI="/proc/pressure/cpu /proc/pressure/memory /proc/pressure/io"
  
  PATHS_SCHED="/proc/sys/kernel/sched_latency_ns /proc/sys/kernel/sched_min_granularity_ns /proc/sys/kernel/sched_wakeup_granularity_ns /proc/sys/kernel/sched_migration_cost_ns /proc/sys/kernel/sched_nr_migrate /proc/sys/kernel/sched_walt_init_task_load_pct /proc/sys/kernel/sched_uclamp_util_min"
  
  PATHS_VM="/proc/sys/vm/swappiness /proc/sys/vm/vfs_cache_pressure /proc/sys/vm/dirty_ratio /proc/sys/vm/dirty_background_ratio /proc/sys/vm/dirty_expire_centisecs /proc/sys/vm/stat_interval /proc/sys/vm/watermark_scale_factor /proc/sys/vm/extfrag_threshold /proc/sys/vm/dirty_writeback_centisecs /proc/sys/vm/page-cluster"
  
  PATHS_STORAGE="/sys/block/mmcblk0/queue/read_ahead_kb /sys/block/mmcblk0/queue/nr_requests /sys/block/mmcblk0/queue/iosched/fifo_batch /sys/block/mmcblk0/stat"
  
  PATHS_THERMAL="/sys/class/thermal/thermal_zone3/temp /sys/class/power_supply/battery/temp /sys/class/power_supply/battery/capacity"
  
  PATHS_INFO="/proc/vmstat /proc/buddyinfo"

  ALL_PATHS="$PATHS_PSI $PATHS_SCHED $PATHS_VM $PATHS_STORAGE $PATHS_THERMAL $PATHS_INFO"

  for path in $ALL_PATHS; do
    if [ ! -e "$path" ]; then
      MISSING_COUNT=$((MISSING_COUNT + 1))
      MISSING_LIST="$MISSING_LIST\n    [x] $path"
    fi
  done

  if [ $MISSING_COUNT -gt 0 ]; then
    ui_print_warn "FOUND $MISSING_COUNT MISSING PATHS!"
    ui_print_warn "Daemon uses the following hardcoded paths:"
    ui_print "$MISSING_LIST"
    ui_print " "
    ui_print_warn "Module might Panic/Crash if forced."
    ui_print_warn "Some paths are very specific."
    ui_print " "
    
    ui_print_log "Waiting for volume key confirmation..."
    ui_print "    [+] Vol Up: Force Install (At your own risk)"
    ui_print "    [-] Vol Down: Cancel (Safe)"
    
    if chooseport; then
      ui_print_info "User selected: CONTINUE"
      ui_print_info "Daemon might need recompilation for this device."
    else
      ui_print_err "User selected: CANCEL"
      abort
    fi
  else
    ui_print_info "All system paths validated."
  fi
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
  if [ -f "$TMPDIR/config.ini.bak" ]; then
    ui_print_log "Restoring user configuration..."
    cp -f "$TMPDIR/config.ini.bak" "$MODPATH/config.ini"
    ui_print_info "Config.ini restored successfully."
  else
    ui_print_log "Using default configuration."
  fi
}

ui_print_header

if [ "$ARCH" != "arm64" ]; then
  ui_print_warn "Architecture $ARCH might be incompatible (Daemon targets arm64)."
fi

check_strict_paths

backup_config
HAS_BACKUP=$?

ui_print_log "Extracting module files..."
unzip -o "$ZIPFILE" 'service.sh' 'system/bin/qos_daemon' 'config.ini' -d "$MODPATH" >&2
unzip -o "$ZIPFILE" 'common/*' -d "$MODPATH" >&2

if [ $HAS_BACKUP -eq 0 ]; then
  restore_config
fi

ui_print_log "Setting permissions..."
set_perm_recursive "$MODPATH" 0 0 0755 0644
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
rm -f "$MODPATH/customize.sh" 2>/dev/null
find "$MODPATH" -empty -type d -delete
[ -e /data/system/package_cache ] && rm -rf /data/system/package_cache/*