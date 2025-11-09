#!/system/bin/sh

if [ "$(ls -A "$MODPATH"/common/addon/*/install.sh 2>/dev/null)" ]; then
  ui_print "- Running Addons"
  for addon in "$MODPATH"/common/addon/*/install.sh; do
    . "$addon"
  done
fi

export_location BootAnimation '/system/product/media/bootanimation.zip'

for addin in "$MODPATH"/common/*.sh; do
  if [ "$addin" != "$MODPATH"/common/install.sh ]; then
    . "$addin"
  fi
done
