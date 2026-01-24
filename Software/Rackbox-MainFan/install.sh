#!/bin/bash

# Compila o programa
echo "Compiling rackfan-daemon..."
cargo build --release

# Cria diretórios
sudo mkdir -p /etc/rackfan
sudo mkdir -p /usr/local/bin

# Instala binário
sudo cp target/release/rackfan-daemon /usr/local/bin/
sudo chmod +x /usr/local/bin/rackfan-daemon

# Instala arquivo de configuração
if [ ! -f /etc/rackfan/config.toml ]; then
    sudo cp config.toml.example /etc/rackfan/config.toml
    echo "Please edit /etc/rackfan/config.toml with your settings"
fi

# Instala serviço systemd
sudo cp rackfan.service /etc/systemd/system/
sudo systemctl daemon-reload

echo "Installation complete!"
echo "Enable service with: sudo systemctl enable rackfan"
echo "Start service with: sudo systemctl start rackfan"
echo "View logs with: sudo journalctl -u rackfan -f"

