#!/bin/bash
set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database /usr/share/mime || true
fi

# Register protocol handler
if command -v xdg-mime &> /dev/null; then
    xdg-mime default tachikoma.desktop x-scheme-handler/tachikoma || true
fi

# Create symlink in /usr/bin
ln -sf /opt/Tachikoma/tachikoma /usr/bin/tachikoma || true

# Set capabilities for sandbox (if needed)
# setcap cap_net_bind_service=+ep /opt/Tachikoma/tachikoma || true

echo "Tachikoma installed successfully!"
exit 0