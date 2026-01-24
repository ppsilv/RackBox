#!/bin/bash
#

set -e

echo "Copiando o daemon"
sudo cp target/release/rackfan_daemon /usr/local/bin/.

echo "Copiando rackfan.service "
# Copie o arquivo de serviço
sudo cp etc/systemd/system/rackfan.service /etc/systemd/system/.

echo "Reloading the daemons.."
# Recarregue systemd
sudo systemctl daemon-reload

echo "Enabling rackboxMainFan"
# Habilite para iniciar automaticamente
sudo systemctl enable rackfan

echo "Starting daemon rackbox-Mainfan"
# Inicie o serviço
sudo systemctl start rackfan

# Verifique logs
sudo journalctl -u rackfan -f
