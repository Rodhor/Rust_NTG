#!/bin/sh
# Print the SHA256 fingerprint of an SSH host key as 64 lowercase hex chars,
# which is the format SFTP_HOST_KEY_SHA256 expects.
#
#   sftp-fingerprint [key.pub]     (default: /etc/ssh/ssh_host_ed25519_key.pub)
#
# Why the padding step: `ssh-keygen -lf` prints the digest as unpadded base64
# (43 chars for 32 bytes). GNU `base64 -d` decodes those 32 bytes correctly but
# then complains "invalid input" about the absent padding, so the output looks
# like a failure. Padding to a multiple of 4 first makes it decode cleanly.
#
# Equivalent to, and cross-checked against:
#   awk '{print $2}' key.pub | base64 -d | sha256sum
set -eu

KEY="${1:-/etc/ssh/ssh_host_ed25519_key.pub}"

B64=$(ssh-keygen -lf "$KEY" | awk '{print $2}' | sed 's/^SHA256://')
PADDED=$(printf '%s' "$B64" | awk '{ n = (4 - length($0) % 4) % 4; printf "%s%s", $0, substr("====", 1, n) }')

fingerprint=$(printf '%s' "$PADDED" | base64 -d | od -An -v -tx1 | tr -d ' \n')

if [ "${#fingerprint}" -ne 64 ]; then
    echo "sftp-fingerprint: expected 64 hex chars from $KEY, got ${#fingerprint}" >&2
    exit 1
fi

echo "$fingerprint"
