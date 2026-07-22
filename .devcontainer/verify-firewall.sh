#!/usr/bin/env bash
# Smoke test for the firewalled dev container.
#
# Run this INSIDE the dev container after "Dev Containers: Rebuild Container":
#     bash .devcontainer/verify-firewall.sh
#
# Exits non-zero if any check fails, so it doubles as a CI/health gate.
set -uo pipefail

pass=0; fail=0
ok() { printf '  \033[32m✓\033[0m %s\n' "$1"; pass=$((pass + 1)); }
no() { printf '  \033[31m✗\033[0m %s\n' "$1"; fail=$((fail + 1)); }

echo "== 1. this container CANNOT change the firewall (tamper-proof) =="
capeff=$(awk '/CapEff/{print $2}' /proc/self/status)
# CAP_NET_ADMIN is capability bit 12.
if python3 -c "import sys; sys.exit(0 if not (int('$capeff',16) >> 12) & 1 else 1)" 2>/dev/null; then
    ok "CAP_NET_ADMIN absent from the dev container"
else
    no "CAP_NET_ADMIN present — rules are tamperable (check compose cap_add/security_opt)"
fi
if sudo -n iptables -A OUTPUT -p tcp --dport 9 -j ACCEPT 2>/dev/null; then
    no "was able to ADD an iptables rule — tamperable!"
    sudo -n iptables -D OUTPUT -p tcp --dport 9 -j ACCEPT 2>/dev/null || true
else
    ok "iptables modification is denied (EPERM or not installed)"
fi

echo "== 2. allow-listed egress works =="
for url in https://pypi.org/simple/ https://registry.npmjs.org/ https://static.crates.io/; do
    if curl -fsS --max-time 15 -o /dev/null "$url"; then ok "reach $url"; else no "cannot reach allow-listed $url"; fi
done

echo "== 3. everything else is blocked =="
# A non-allow-listed HTTPS host should be dropped (DNS resolves, TCP is denied).
if curl -fsS --max-time 8 -o /dev/null https://example.com 2>/dev/null; then
    no "reached non-allow-listed https://example.com — allowlist NOT enforced"
else
    ok "non-allow-listed https://example.com blocked"
fi
# Outbound SMB must be blocked (default-deny -> timeout, or REJECT -> refused).
if timeout 6 bash -c 'exec 3<>/dev/tcp/1.1.1.1/445' 2>/dev/null; then
    no "outbound SMB (tcp/445) allowed"
    exec 3>&- 2>/dev/null || true
else
    ok "outbound SMB (tcp/445) blocked"
fi

echo "== 4. loopback / vite dev server path works =="
python3 -m http.server 5173 --bind 127.0.0.1 >/dev/null 2>&1 &
srv=$!
sleep 1
if curl -fsS --max-time 5 -o /dev/null http://127.0.0.1:5173/; then
    ok "localhost:5173 reachable (vite serve + HMR path)"
else
    no "loopback dev server unreachable"
fi
kill "$srv" 2>/dev/null || true

echo
echo "  passed: $pass    failed: $fail"
if [ "$fail" -ne 0 ]; then
    echo "  ✗ FIREWALL VERIFICATION FAILED"
    exit 1
fi
echo "  ✓ firewall verification passed"
