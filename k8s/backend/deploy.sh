#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NAMESPACE="apptolast-inemsellar"

echo "Deploying inem-sellar-backend to namespace: $NAMESPACE"

echo "[1/8] Applying ConfigMap..."
kubectl apply -f "$SCRIPT_DIR/configmap.yaml"

echo "[2/8] Applying Secret..."
kubectl apply -f "$SCRIPT_DIR/secret.yaml"

echo "[3/8] Applying Deployment..."
kubectl apply -f "$SCRIPT_DIR/deployment.yaml"

echo "[4/8] Applying Service..."
kubectl apply -f "$SCRIPT_DIR/service.yaml"

echo "[5/8] Applying Certificate..."
kubectl apply -f "$SCRIPT_DIR/certificate.yaml"

echo "[6/8] Applying IngressRoute..."
kubectl apply -f "$SCRIPT_DIR/ingressroute.yaml"

echo "[7/8] Applying Middleware..."
kubectl apply -f "$SCRIPT_DIR/middleware.yaml"

echo "[8/8] Applying CronJob image-updater..."
kubectl apply -f "$SCRIPT_DIR/cronjob-image-updater.yaml"

echo "Waiting for deployment to be ready..."
kubectl rollout status deployment/inem-sellar-backend -n "$NAMESPACE" --timeout=5m

echo "Deploy completed successfully."
kubectl get pods -n "$NAMESPACE" -l app=inem-sellar-backend
