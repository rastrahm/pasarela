#!/usr/bin/env bash
# Provisiona un VPS Ubuntu/Debian para staging (Fase 7.1).
# Ejecutar como root en el servidor recién creado:
#   curl -fsSL ... | bash   # o scp + bash
#   sudo bash deploy/staging/provision-vps.sh
#
# Instala: Docker Engine + Compose plugin, UFW (22/80/443), usuario deploy opcional.
set -euo pipefail

DEPLOY_USER="${DEPLOY_USER:-deploy}"
INSTALL_USER=true
SKIP_UFW=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-ufw) SKIP_UFW=true; shift ;;
    --no-deploy-user) INSTALL_USER=false; shift ;;
    -h|--help)
      sed -n '1,8p' "$0"
      exit 0
      ;;
    *) echo "Argumento desconocido: $1" >&2; exit 2 ;;
  esac
done

if [[ "$(id -u)" -ne 0 ]]; then
  echo "ERROR: ejecutar como root (sudo)." >&2
  exit 1
fi

echo "=== provision-vps — Pasarela staging ==="

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq ca-certificates curl gnupg ufw git

if ! command -v docker >/dev/null 2>&1; then
  echo "Instalando Docker Engine..."
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
    | gpg --dearmor -o /etc/apt/keyrings/docker.gpg 2>/dev/null \
    || curl -fsSL https://download.docker.com/linux/debian/gpg \
    | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg

  . /etc/os-release
  echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/${ID} ${VERSION_CODENAME} stable" \
    > /etc/apt/sources.list.d/docker.list

  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
  systemctl enable --now docker
  echo "  ✓ Docker instalado: $(docker --version)"
else
  echo "  ✓ Docker ya presente: $(docker --version)"
fi

if [[ "$SKIP_UFW" != true ]]; then
  echo "Configurando UFW (22, 80, 443)..."
  ufw default deny incoming
  ufw default allow outgoing
  ufw allow 22/tcp comment 'SSH'
  ufw allow 80/tcp comment 'HTTP ACME + redirect'
  ufw allow 443/tcp comment 'HTTPS Caddy'
  ufw --force enable
  echo "  ✓ UFW activo — Oracle/Gateway NO expuestos en host"
fi

if [[ "$INSTALL_USER" == true ]]; then
  if ! id "$DEPLOY_USER" &>/dev/null; then
    useradd -m -s /bin/bash "$DEPLOY_USER"
    usermod -aG docker "$DEPLOY_USER"
    echo "  ✓ Usuario ${DEPLOY_USER} creado (grupo docker)"
    echo "    Configurá SSH key: ssh-copy-id ${DEPLOY_USER}@<vps-ip>"
  else
    usermod -aG docker "$DEPLOY_USER" 2>/dev/null || true
    echo "  ✓ Usuario ${DEPLOY_USER} ya existe"
  fi
fi

echo
echo "=== Provision OK ==="
echo "Siguiente:"
echo "  1. DNS A/AAAA → IP de este VPS"
echo "  2. git clone <repo> && cd pasarela"
echo "  3. ./scripts/generate-staging-secrets.sh"
echo "  4. Editar deploy/staging/.env (dominio, devnet)"
echo "  5. ./scripts/deploy-staging.sh"
echo
echo "Ver Doc/Provision-VPS-Fase-7.md"
