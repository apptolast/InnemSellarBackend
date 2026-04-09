#!/bin/bash
set -euo pipefail

# Despliegue PostgreSQL para InemSellar en Kubernetes
# Namespace: apptolast-inemsellar
# NodePort: 30435

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
NAMESPACE="apptolast-inemsellar"

echo "=== Desplegando PostgreSQL para InemSellar ==="

# 1. Namespace
echo "[1/7] Creando namespace..."
kubectl apply -f "$SCRIPT_DIR/namespace.yaml"

# 2. Secret
echo "[2/7] Creando secret..."
kubectl apply -f "$SCRIPT_DIR/secret.yaml"

# 3. ConfigMap con schema.sql
echo "[3/7] Creando ConfigMap con schema.sql..."
kubectl create configmap postgres-init-schema \
  --from-file=schema.sql="$PROJECT_DIR/schema.sql" \
  --namespace="$NAMESPACE" \
  --dry-run=client -o yaml | kubectl apply -f -

# 4. PVC
echo "[4/7] Creando PersistentVolumeClaim..."
kubectl apply -f "$SCRIPT_DIR/pvc.yaml"

# 5. StatefulSet
echo "[5/7] Creando StatefulSet..."
kubectl apply -f "$SCRIPT_DIR/statefulset.yaml"

# 6. Services
echo "[6/7] Creando Services..."
kubectl apply -f "$SCRIPT_DIR/service.yaml"
kubectl apply -f "$SCRIPT_DIR/service-external.yaml"

# 7. Esperar a que el pod este Ready
echo "[7/7] Esperando a que postgres-0 este Ready..."
kubectl wait --for=condition=ready pod/postgres-0 \
  --namespace="$NAMESPACE" \
  --timeout=120s

echo ""
echo "=== Despliegue completado ==="
echo "  Namespace:  $NAMESPACE"
echo "  Pod:        postgres-0"
echo "  DB:         inemsellar"
echo "  Usuario:    admin"
echo "  ClusterIP:  postgres.$NAMESPACE.svc.cluster.local:5432"
echo "  NodePort:   138.199.157.58:30435"
echo ""
echo "Conexion externa:"
echo "  psql -h 138.199.157.58 -p 30435 -U admin -d inemsellar"
