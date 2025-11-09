#!/system/bin/sh

if [ -f "$BootAnimation_location" ]; then
    ui_print ' [+] Installed Boot-Animation.'
    ui_print ''
else
  ui_print " [x] Resources missing !"
  touch "$MODPATH"/remove
  ui_print " [---] Module will be removed on the next boot."
fi
