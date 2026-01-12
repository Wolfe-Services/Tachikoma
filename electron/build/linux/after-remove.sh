#!/bin/bash
# build/linux/after-remove.sh

# Remove symlink
rm -f /usr/local/bin/tachikoma || true

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

echo "Tachikoma uninstallation complete!"