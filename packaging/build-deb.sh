#!/bin/bash
# Build DEB package for LucAstra

set -e

VERSION="${1:-1.0.0}"
ARCH="amd64"
PKG_NAME="lucastra"
PKG_DIR="${PKG_NAME}_${VERSION}_${ARCH}"

echo "Building DEB package: ${PKG_DIR}"

# Create package structure
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/local/bin"
mkdir -p "${PKG_DIR}/etc/lucastra"
mkdir -p "${PKG_DIR}/var/log/lucastra"
mkdir -p "${PKG_DIR}/var/lib/lucastra/audit"
mkdir -p "${PKG_DIR}/var/lib/lucastra/metrics"
mkdir -p "${PKG_DIR}/usr/share/doc/lucastra"

# Copy binary
cp ../target/release/app "${PKG_DIR}/usr/local/bin/lucastra"
chmod 755 "${PKG_DIR}/usr/local/bin/lucastra"

# Copy default config
cp ../docs/examples/configs/prod.json "${PKG_DIR}/etc/lucastra/config.json"

# Copy documentation
cp ../README.md "${PKG_DIR}/usr/share/doc/lucastra/"
cp ../docs/CHANGELOG.md "${PKG_DIR}/usr/share/doc/lucastra/"
cp ../docs/RELEASE_NOTES.md "${PKG_DIR}/usr/share/doc/lucastra/"

# Create control file
cat > "${PKG_DIR}/DEBIAN/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: misc
Priority: optional
Architecture: ${ARCH}
Maintainer: LucAstra Team <team@lucastra.dev>
Description: Augmented operating system with embedded LLM
 LucAstra is a prototype operating system that deeply integrates
 a local 7B parameter language model for natural language interaction,
 intelligent search (BM25), and autonomous task execution via tools.
 Everything runs locally for privacy and control.
Depends: libc6 (>= 2.31), libssl3 (>= 3.0.0)
EOF

# Create postinst script
cat > "${PKG_DIR}/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e

# Create lucastra user if doesn't exist
if ! id -u lucastra > /dev/null 2>&1; then
    useradd -r -s /bin/false -d /var/lib/lucastra lucastra
fi

# Set ownership
chown -R lucastra:lucastra /var/log/lucastra
chown -R lucastra:lucastra /var/lib/lucastra
chown lucastra:lucastra /etc/lucastra/config.json

echo "LucAstra installed successfully!"
echo "Default config: /etc/lucastra/config.json"
echo "Set LUCASTRA_CONFIG_HOME=/etc/lucastra to use system config"
EOF

chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# Create postrm script
cat > "${PKG_DIR}/DEBIAN/postrm" <<'EOF'
#!/bin/bash
set -e

if [ "$1" = "purge" ]; then
    # Remove logs and data on purge
    rm -rf /var/log/lucastra
    rm -rf /var/lib/lucastra
    rm -rf /etc/lucastra
    
    # Remove user
    if id -u lucastra > /dev/null 2>&1; then
        userdel lucastra || true
    fi
fi
EOF

chmod 755 "${PKG_DIR}/DEBIAN/postrm"

# Build package
dpkg-deb --build "${PKG_DIR}"

echo "DEB package created: ${PKG_DIR}.deb"
echo "Install with: sudo dpkg -i ${PKG_DIR}.deb"
