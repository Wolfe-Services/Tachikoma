#!/bin/bash
# scripts/sign-linux.sh

set -e

# Configuration
GPG_KEY_ID="${GPG_KEY_ID:-}"
PACKAGE_DIR="${1:-release}"

if [ -z "$GPG_KEY_ID" ]; then
    echo "GPG_KEY_ID not set, skipping signing"
    exit 0
fi

echo "Signing Linux packages with GPG key: $GPG_KEY_ID..."

# Check if GPG key exists
if ! gpg --list-secret-keys "$GPG_KEY_ID" >/dev/null 2>&1; then
    echo "Error: GPG key $GPG_KEY_ID not found in keyring"
    exit 1
fi

# Make sure package directory exists
if [ ! -d "$PACKAGE_DIR" ]; then
    echo "Package directory $PACKAGE_DIR not found"
    exit 1
fi

# Sign DEB packages
for deb in "$PACKAGE_DIR"/*.deb; do
    if [ -f "$deb" ]; then
        echo "Signing DEB package: $deb"
        
        # Check if dpkg-sig is available
        if command -v dpkg-sig >/dev/null 2>&1; then
            dpkg-sig -k "$GPG_KEY_ID" --sign builder "$deb"
            dpkg-sig --verify "$deb"
        else
            echo "Warning: dpkg-sig not found, skipping DEB signing"
        fi
    fi
done

# Sign RPM packages  
for rpm in "$PACKAGE_DIR"/*.rpm; do
    if [ -f "$rpm" ]; then
        echo "Signing RPM package: $rpm"
        
        # Check if rpm command supports signing
        if command -v rpm >/dev/null 2>&1; then
            # Create rpmmacros if it doesn't exist
            if [ ! -f "$HOME/.rpmmacros" ]; then
                cat > "$HOME/.rpmmacros" << EOF
%_gpg_name $GPG_KEY_ID
%_signature gpg
%_gpg_path ~/.gnupg
%__gpg /usr/bin/gpg
%__gpg_sign_cmd %{__gpg} --batch --no-verbose --no-armor --no-secmem-warning -u "%{_gpg_name}" -sbo %{__signature_filename} %{__plaintext_filename}
EOF
            fi
            
            rpm --addsign "$rpm"
            rpm --checksig "$rpm"
        else
            echo "Warning: rpm command not found, skipping RPM signing"
        fi
    fi
done

# Sign AppImage files
for appimage in "$PACKAGE_DIR"/*.AppImage; do
    if [ -f "$appimage" ]; then
        echo "Creating signature for AppImage: $appimage"
        gpg --armor --detach-sign --default-key "$GPG_KEY_ID" "$appimage"
        echo "Created signature: $appimage.asc"
    fi
done

# Create checksums and sign them
echo "Creating checksums..."
cd "$PACKAGE_DIR"

# Create SHA256 checksums
sha256sum *.AppImage *.deb *.rpm *.tar.gz *.snap 2>/dev/null | sort > SHA256SUMS.txt || true

# Sign the checksums
if [ -f "SHA256SUMS.txt" ]; then
    gpg --armor --detach-sign --default-key "$GPG_KEY_ID" SHA256SUMS.txt
    echo "Created signed checksums: SHA256SUMS.txt.asc"
fi

cd - >/dev/null

echo "Linux package signing complete!"

# Verify all signatures
echo "Verifying signatures..."
for sig in "$PACKAGE_DIR"/*.asc; do
    if [ -f "$sig" ]; then
        echo "Verifying: $sig"
        gpg --verify "$sig"
    fi
done