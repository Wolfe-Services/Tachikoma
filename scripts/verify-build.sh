#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
ARTIFACTS_DIR="${2:-electron/out}"

echo "=== Build Verification for v${VERSION} ==="

ERRORS=0

# Function to report errors
report_error() {
    echo "ERROR: $1"
    ((ERRORS++))
}

# Function to report success
report_ok() {
    echo "OK: $1"
}

# Check all expected artifacts exist
check_artifacts() {
    echo ""
    echo "--- Checking Artifacts ---"

    local expected_files=(
        "Tachikoma-${VERSION}-arm64.dmg"
        "Tachikoma-${VERSION}-x64.dmg"
        "Tachikoma-Setup-${VERSION}.exe"
        "Tachikoma-${VERSION}.AppImage"
        "tachikoma_${VERSION}_amd64.deb"
        "latest-mac.yml"
        "latest.yml"
        "latest-linux.yml"
    )

    for file in "${expected_files[@]}"; do
        if [ -f "${ARTIFACTS_DIR}/${file}" ]; then
            report_ok "Found ${file}"
        else
            report_error "Missing ${file}"
        fi
    done
}

# Verify file sizes are reasonable
check_file_sizes() {
    echo ""
    echo "--- Checking File Sizes ---"

    local min_size_mb=50  # Minimum expected size in MB

    for file in "${ARTIFACTS_DIR}"/*.{dmg,exe,AppImage,deb} 2>/dev/null; do
        if [ -f "$file" ]; then
            local size_mb=$(du -m "$file" | cut -f1)
            local filename=$(basename "$file")

            if [ "$size_mb" -lt "$min_size_mb" ]; then
                report_error "${filename} is too small (${size_mb}MB < ${min_size_mb}MB)"
            else
                report_ok "${filename} size OK (${size_mb}MB)"
            fi
        fi
    done
}

# Verify checksums can be generated
check_checksums() {
    echo ""
    echo "--- Generating Checksums ---"

    local checksum_file="${ARTIFACTS_DIR}/checksums.sha256"

    shasum -a 256 "${ARTIFACTS_DIR}"/*.{dmg,exe,AppImage,deb} 2>/dev/null > "$checksum_file" || true

    if [ -s "$checksum_file" ]; then
        report_ok "Checksums generated"
        cat "$checksum_file"
    else
        report_error "Failed to generate checksums"
    fi
}

# Main
check_artifacts
check_file_sizes
check_checksums

echo ""
echo "=== Verification Complete ==="

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s) found"
    exit 1
else
    echo "PASSED: All checks passed"
fi