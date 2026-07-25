#!/usr/bin/env bash
# Crea la base de datos PostgreSQL local para el Oracle.
# Uso: ./scripts/setup-db.sh [nombre_bd] [usuario]
set -euo pipefail

DB_NAME="${1:-oracle}"
DB_USER="${2:-postgres}"

echo "Creando base de datos '${DB_NAME}' (usuario: ${DB_USER})..."
psql -U "${DB_USER}" -h localhost -tc "SELECT 1 FROM pg_database WHERE datname='${DB_NAME}'" | grep -q 1 \
  || psql -U "${DB_USER}" -h localhost -c "CREATE DATABASE ${DB_NAME};"

echo "Listo. Configura ORACLE_DATABASE_URL en .env, por ejemplo:"
echo "  ORACLE_DATABASE_URL=postgres://${DB_USER}:<password>@localhost:5432/${DB_NAME}"
