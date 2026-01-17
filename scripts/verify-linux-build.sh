#!/usr/bin/env bash
set -euo pipefail

ARTIFACT="${1:?Artifact path required}"
ARTIFACT_TYPE="${2:-auto}"

echo "=== Linux Build Verification ==="
echo "File: ${ARTIFACT}"

ERRORS=0

# Auto-detect type
if [ "$ARTIFACT_TYPE" = "auto" ]; then
    case "$ARTIFACT" in
        *.AppImage) ARTIFACT_TYPE="appimage" ;;
        *.deb) ARTIFACT_TYPE="deb" ;;
        *) echo "ERROR: Unknown artifact type"; exit 1 ;;
    esac
fi

verify_appimage() {
    local appimage="$1"
    echo ""
    echo "--- Verifying AppImage ---"

    # Check it's executable
    if [ -x "$appimage" ]; then
        echo "OK: File is executable"
    else
        chmod +x "$appimage"
        echo "OK: Made executable"
    fi

    # Check ELF header
    if file "$appimage" | grep -q "ELF"; then
        echo "OK: Valid ELF binary"
    else
        echo "ERROR: Not a valid ELF binary"
        ((ERRORS++))
    fi

    # Check AppImage type
    if file "$appimage" | grep -q "AppImage"; then
        echo "OK: AppImage format detected"
    fi

    # Extract and verify contents
    local extract_dir="/tmp/appimage-verify-$$"
    mkdir -p "$extract_dir"

    echo ""
    echo "--- Extracting AppImage ---"
    cd "$extract_dir"
    "$appimage" --appimage-extract > /dev/null 2>&1 || {
        echo "ERROR: Failed to extract AppImage"
        ((ERRORS++))
        return
    }

    # Check required files
    local required_files=(
        "squashfs-root/tachikoma"
        "squashfs-root/resources/app.asar"
        "squashfs-root/tachikoma.desktop"
    )

    for file in "${required_files[@]}"; do
        if [ -e "$file" ]; then
            echo "OK: Found $file"
        else
            echo "ERROR: Missing $file"
            ((ERRORS++))
        fi
    done

    # Check desktop file
    if [ -f "squashfs-root/tachikoma.desktop" ]; then
        echo ""
        echo "--- Desktop Entry ---"
        cat "squashfs-root/tachikoma.desktop" | head -20
    fi

    # Cleanup
    rm -rf "$extract_dir"
}

verify_deb() {
    local deb="$1"
    echo ""
    echo "--- Verifying Debian Package ---"

    # Check package info
    echo ""
    echo "--- Package Info ---"
    dpkg-deb --info "$deb"

    # Check package contents
    echo ""
    echo "--- Package Contents ---"
    dpkg-deb --contents "$deb" | head -30

    # Extract control file
    local extract_dir="/tmp/deb-verify-$$"
    mkdir -p "$extract_dir"
    dpkg-deb --control "$deb" "$extract_dir"

    # Check control file
    if [ -f "$extract_dir/control" ]; then
        echo ""
        echo "--- Control File ---"
        cat "$extract_dir/control"

        # Verify required fields
        for field in Package Version Architecture Maintainer Description; do
            if grep -q "^$field:" "$extract_dir/control"; then
                echo "OK: Has $field field"
            else
                echo "ERROR: Missing $field field"
                ((ERRORS++))
            fi
        done
    fi

    # Check dependencies
    echo ""
    echo "--- Dependencies ---"
    dpkg-deb --field "$deb" Depends

    # Check for post-install script
    if [ -f "$extract_dir/postinst" ]; then
        echo ""
        echo "OK: Has post-install script"
    fi

    # Cleanup
    rm -rf "$extract_dir"
}

case "$ARTIFACT_TYPE" in
    appimage) verify_appimage "$ARTIFACT" ;;
    deb) verify_deb "$ARTIFACT" ;;
esac

echo ""
echo "=== Linux Verification Complete ==="

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s)"
    exit 1
else
    echo "PASSED"
fi