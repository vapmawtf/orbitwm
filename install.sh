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

echo "📦 Installing OrbitWM..."
sudo install -m 0755 ./target/release/orbitwm /usr/local/bin/orbitwm.new
sudo mv -f /usr/local/bin/orbitwm.new /usr/local/bin/orbitwm
echo "✅ OrbitWM updated successfully!"