#!/system/bin/sh

SKIPUNZIP=1
MODID="sys_qos"

unzip -o "$ZIPFILE" 'module.prop' 'common/utils.sh' 'common/setup.sh' -d "$MODPATH" >&2

. "$MODPATH/common/utils.sh"
. "$MODPATH/common/setup.sh"

ui_print_header

ui_print_log "[1/8] Verifying system environment..."
if [ "$ARCH" != "arm64" ]; then
    ui_print_warn "Incompatible architecture ($ARCH). Daemon targets arm64."
fi

ui_print_log "[2/8] Checking previous installation..."
backup_config
HAS_BACKUP=$?

ui_print_log "[3/8] Extracting module files..."
unzip -o "$ZIPFILE" 'service.sh' 'system/bin/qos_daemon' 'config.ini' 'system.prop' 'common/apk/*' 'uninstall.sh' -d "$MODPATH" >&2

ui_print_log "[4/8] Validating system properties..."
validate_system_props

ui_print_log "[5/8] Configuring module settings..."

if [ $HAS_BACKUP -eq 0 ]; then
    ui_print_warn "Existing configuration found."
    ui_print "  (+) Vol Up   : MERGE (Keep old settings)"
    ui_print "  (-) Vol Down : RESET (Start fresh)"
    
    if chooseport; then
        restore_config
    else
        ui_print_info "Old configuration discarded."
        run_setup_wizard
    fi
else
    ui_print_info "Fresh installation detected."
    ui_print "  (+) Vol Up   : CUSTOMIZE (Choose features)"
    ui_print "  (-) Vol Down : DEFAULT                    "
    
    if chooseport; then
        run_setup_wizard
    else
        ui_print_info "Applying default configuration."
    fi
fi

ui_print " "
ui_print_log "[6/8] Setting up Application..."
install_app_wizard

ui_print_log "[7/8] Setting file permissions..."
set_perm_recursive "$MODPATH" 0 0 0755 0644
set_perm "$MODPATH/service.sh" 0 0 0755
set_perm "$MODPATH/system/bin/qos_daemon" 0 0 0755
set_perm "$MODPATH/config.ini" 0 0 0644

ui_print_log "[8/8] Finalizing installation..."
rm -f "$MODPATH/common/utils.sh" "$MODPATH/common/setup.sh" "$MODPATH/customize.sh" "$MODPATH/update.json" 2>/dev/null
find "$MODPATH" -empty -type d -delete
[ -e /data/system/package_cache ] && rm -rf /data/system/package_cache/*
