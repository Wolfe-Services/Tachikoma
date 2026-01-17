#!/bin/bash
set -e

case "$1" in
    purge|remove)
        # Remove symlink
        rm -f /usr/bin/tachikoma || true

        # Update databases
        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database -q /usr/share/applications || true
        fi

        if command -v gtk-update-icon-cache &> /dev/null; then
            gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
        fi

        if command -v update-mime-database &> /dev/null; then
            update-mime-database /usr/share/mime || true
        fi

        # Remove config directory on purge
        if [ "$1" = "purge" ]; then
            rm -rf /etc/tachikoma || true
        fi
        ;;
esac

exit 0