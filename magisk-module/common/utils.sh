#!/system/bin/sh

ACTIVE_DIR="/data/adb/modules/$MODID"

grep_prop() { sed -n "s/^$1=//p" "$2"; }
get_prop() { getprop "$1"; }

ui_print_header() {
    ui_print "***********************************************"
    ui_print "  Version : v$(grep_prop version "$MODPATH/module.prop")"
    ui_print "***********************************************"
}

ui_print_log() { ui_print "- $1"; }
ui_print_info() { ui_print "  ➜ $1"; }
ui_print_warn() { ui_print "  ! $1"; }
ui_print_err() { ui_print "  X $1"; }

# Original concept by Chainfire, modernized for new Android versions
chooseport() {
    sleep 0.5
    timeout 0.2 /system/bin/getevent -l -c 1 > /dev/null 2>&1
    
    while true; do
        timeout 15 /system/bin/getevent -l -c 1 > "$TMPDIR/events" 2>/dev/null
        
        if [ -s "$TMPDIR/events" ]; then
            if grep -qE "KEY_VOLUMEUP| 0073 " "$TMPDIR/events"; then return 0; fi
            if grep -qE "KEY_VOLUMEDOWN| 0072 " "$TMPDIR/events"; then return 1; fi
        fi
    done
}
