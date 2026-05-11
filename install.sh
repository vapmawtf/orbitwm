#!/usr/bin/env bash
set -e

echo "🔨 Building OrbitWM..."
cargo build --release

echo "📦 Installing Configs..."
sudo mkdir -p /usr/share/xsessions
mkdir -p ~/.config/polybar
sudo cp ./config/orbitwm.desktop /usr/share/xsessions/orbitwm.desktop
sudo mkdir -p /etc/orbitwm
sudo cp ./config/config.toml /etc/orbitwm/config.toml

cp ./config/polybar/config.ini ~/.config/polybar/config.ini
cp ./config/polybar/launch.sh ~/.config/polybar/launch.sh
chmod +x ~/.config/polybar/launch.sh

echo "🛑 Stopping OrbitWM..."
pkill orbitwm 2>/dev/null || true
sleep 0.5
echo "📦 Installing OrbitWM..."
sudo cp ./target/release/orbitwm /usr/local/bin/orbitwm
echo "✅ OrbitWM updated successfully!"