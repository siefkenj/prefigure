#!/usr/bin/env bash
# Entrypoint for the firewall sidecar. Applies the baked allowlist ruleset to
# the shared network namespace, then stays alive so the dev container keeps its
# network. Runs as root here (NET_ADMIN comes from compose cap_add), so no sudo.
set -euo pipefail

# Apply the feature's default-deny + allowlist ruleset.
/usr/local/bin/init-firewall.sh

# Allow inbound to the dev-server ports so the host can hit them directly
# (VS Code's own forwarding already works over loopback inside the shared netns;
# this is only for http://localhost:5173 straight from your browser). Inserted
# at the top so it beats the feature's default-DROP on INPUT.
for p in 5173 4173 3000 8080; do
    iptables -I INPUT -p tcp --dport "$p" -j ACCEPT || true
done

# Signal readiness for the compose healthcheck (dev waits on this).
mkdir -p /run && touch /run/fw-ready
echo "[firewall] ruleset applied; egress restricted to the allowlist."

# Own the namespace for the container's lifetime.
exec sleep infinity
