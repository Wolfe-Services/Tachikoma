#!/bin/bash
# build/linux/after-install.sh

set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

# Create symlink in /usr/local/bin
ln -sf /opt/Tachikoma/tachikoma /usr/local/bin/tachikoma || true

echo "Tachikoma installation complete!"