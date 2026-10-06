#!/bin/bash
# One-time setup of the verification box (tools/vbox.py). Run on the box as
# `ubuntu`: `bash vbox_setup.sh` (tools/vbox.py setup copies and runs it).
# Idempotent: rerunning it upgrades nothing that is already in place.
set -euo pipefail

NODE=v20.20.2   # GitHub Actions setup-node "20" at the time of writing (oracle.yml, tier4.yml)

sudo DEBIAN_FRONTEND=noninteractive apt-get update -q
sudo DEBIAN_FRONTEND=noninteractive apt-get install -yq build-essential pkg-config rsync python3 xz-utils curl git

if [ ! -x /opt/node20/bin/node ] || [ "$(/opt/node20/bin/node --version)" != "$NODE" ]; then
  curl -fsSL "https://nodejs.org/dist/$NODE/node-$NODE-linux-x64.tar.xz" -o /tmp/node.tar.xz
  sudo rm -rf /opt/node20 && sudo mkdir -p /opt/node20
  sudo tar -xJf /tmp/node.tar.xz -C /opt/node20 --strip-components=1
  rm /tmp/node.tar.xz
fi

if [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi
"$HOME/.cargo/bin/rustup" update stable

mkdir -p "$HOME/vbox" "$HOME/jobs" "$HOME/builds" "$HOME/stage" "$HOME/nm"

# Every 2 minutes: record running time, enforce the spending limits, and stop
# the instance after 30 idle minutes (no running job, no vbox call, no SSH session) (tools/vbox.py agent idle-check, installed as vbox/agent.py).
sudo tee /etc/systemd/system/vbox-idle.service >/dev/null <<'EOF'
[Unit]
Description=Verification box running time, spending limits and idle stop

[Service]
Type=oneshot
ExecStart=/usr/bin/python3 /home/ubuntu/vbox/agent.py agent idle-check
EOF
sudo tee /etc/systemd/system/vbox-idle.timer >/dev/null <<'EOF'
[Unit]
Description=Record the verification box's running time and stop it when idle or over budget

[Timer]
OnBootSec=1min
OnUnitActiveSec=2min

[Install]
WantedBy=timers.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable --now vbox-idle.timer

/opt/node20/bin/node --version
"$HOME/.cargo/bin/cargo" --version
