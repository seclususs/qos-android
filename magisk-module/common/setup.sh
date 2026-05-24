#!/system/bin/sh

backup_config() {
    if [ -f "$ACTIVE_DIR/config.ini" ]; then
        cp -f "$ACTIVE_DIR/config.ini" "$TMPDIR/config.ini.bak"
        ui_print_info "Previous configuration backed up."
        return 0
    fi
    
    ui_print_info "No previous configuration found."
    return 1
}

restore_config() {
    local OLD_CFG="$TMPDIR/config.ini.bak"
    local NEW_CFG="$MODPATH/config.ini"
    
    if [ -f "$OLD_CFG" ]; then
        ui_print_info "Restoring previous configuration..."
        
        grep '^[a-zA-Z0-9_.]\+=' "$OLD_CFG" | while read -r line; do
            local key=$(echo "$line" | cut -d'=' -f1)
            local val=$(echo "$line" | cut -d'=' -f2-)
            
            if grep -q "^$key=" "$NEW_CFG"; then
                sed -i "s|^$key=.*|$key=$val|" "$NEW_CFG"
            fi
        done
        
        ui_print_info "Configuration merged successfully."
    else
        ui_print_info "Applying default configuration."
    fi
}

run_setup_wizard() {
    ui_print_info "Initializing setup wizard..."
    ui_print " "
    FEATURES="Blocker:blocker_enabled Cleaner:cleaner_enabled CPU_Controller:cpu_enabled Storage_Controller:storage_enabled System_Tweaks:tweaks_enabled"
    
    for item in $FEATURES; do
        local name=$(echo "$item" | cut -d':' -f1 | tr '_' ' ')
        local key=$(echo "$item" | cut -d':' -f2)
        local warning=""
        
        case "$key" in
            "cpu_enabled") [ ! -e "/proc/pressure/cpu" ] && warning="PSI CPU missing. Service might degrade.";;
            "storage_enabled") [ ! -e "/proc/pressure/io" ] && warning="PSI IO missing. Service will fail.";;
            "cleaner_enabled")
                if [ ! -d "/data/data" ] || [ ! -d "/proc" ]; then warning="System paths inaccessible."
            elif [ ! -e "/proc/pressure/cpu" ] || [ ! -e "/proc/pressure/io" ]; then warning="PSI metrics missing."; fi;;
        esac
        
        ui_print "- [?] Enable $name?"
        [ ! -z "$warning" ] && ui_print_warn "$warning"
        ui_print "  (+) Vol Up   = ENABLE"
        ui_print "  (-) Vol Down = DISABLE"
        
        if chooseport; then
            sed -i "s|^$key=.*|$key=true|" "$MODPATH/config.ini"
            ui_print_info "$name -> ENABLED"
        else
            sed -i "s|^$key=.*|$key=false|" "$MODPATH/config.ini"
            ui_print_warn "$name -> DISABLED"
        fi
        
        ui_print " "
        sleep 0.2
    done
    
    ui_print_info "Setup wizard complete."
}

validate_system_props() {
    REQUIRED_PROPS="ro.vendor.mtk.bt_sap_enable ro.vendor.mtk_wappush_support ro.vendor.mtk_c2k_support ro.vendor.mtk_c2k_lte_mode ro.vendor.mtk_embms_support ro.vendor.mtk_md_world_mode_support ro.vendor.connsys.dedicated.log ro.vendor.mtk_protocol1_rat_config ro.vendor.mtk_wapi_support"
    local remove_system_prop=0
    
    for prop in $REQUIRED_PROPS; do
        key=$(echo "$prop" | cut -d'=' -f1)
        
        if [ -z "$(get_prop "$key")" ]; then
            ui_print_warn "Missing property: $key"
            remove_system_prop=1
        fi
    done
    
    if [ $remove_system_prop -eq 1 ] && [ -f "$MODPATH/system.prop" ]; then
        ui_print_warn "Required properties missing. system.prop removed."
        rm -f "$MODPATH/system.prop"
    else
        ui_print_info "System properties validated successfully."
    fi
}

install_app_wizard() {
    ui_print "- [?] Install QoS Application?"
    ui_print "  (+) Vol Up   = INSTALL"
    ui_print "  (-) Vol Down = SKIP"
    
    if chooseport; then
        ui_print_info "Preparing app installation..."
        
        cp -f "$MODPATH/common/apk/installer.sh" "/data/adb/service.d/qos_installer_${MODID}.sh"
        
        set_perm "/data/adb/service.d/qos_installer_${MODID}.sh" 0 0 0755
        set_perm "$MODPATH/uninstall.sh" 0 0 0755
        
        ui_print_info "App will be installed automatically after boot."
    else
        ui_print_info "App installation skipped."
        
        rm -rf "$MODPATH/common/apk" "$MODPATH/uninstall.sh"
    fi
    
    ui_print " "
}
