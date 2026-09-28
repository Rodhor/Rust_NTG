#!/usr/bin/env bash
# Print the SFTP host key fingerprint in the exact format SFTP_HOST_KEY_SHA256 wants.
#
# The host key is generated at image build time, so this value is stable for the
# life of the image. Rebuilding (or --no-cache) produces a new one.
#
# If you do not have this pinned yet, you do not need this script: point
# SFTP_HOST_KEY_SHA256 at any 64 hex characters, call the API, and read the
# real fingerprint out of the HOST_KEY_MISMATCH response body.
set -euo pipefail
cd "$(dirname "$0")"

# Works whether the server is up or only built.
if docker compose ps --status running --services 2>/dev/null | grep -qx sftp; then
    fingerprint=$(docker compose exec -T sftp sftp-fingerprint)
else
    fingerprint=$(docker compose run --rm --no-deps -T sftp sftp-fingerprint)
fi

if [ "${#fingerprint}" -ne 64 ]; then
    echo "error: expected 64 hex characters, got ${#fingerprint}: '$fingerprint'" >&2
    exit 1
fi

cat <<EOF
SFTP_HOST_KEY_SHA256=$fingerprint

Add to your .env, alongside:
  SFTP_HOST=127.0.0.1
  SFTP_PORT=2222
  SFTP_USERNAME=dev
  SFTP_PASSWORD=dev-password
  SFTP_ROOT_DIR=/incoming
EOF
