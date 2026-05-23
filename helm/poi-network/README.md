# POI Network Helm Chart

Deploys the [Decentralized Proof-of-Contact Network](https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network) on Kubernetes.

## Prerequisites

- Kubernetes 1.28+
- Helm 3.10+
- cert-manager (for TLS)
- nginx-ingress-controller
- AWS EBS CSI driver (for PVCs on EKS)

## Installing

```bash
# Add the namespace
kubectl create namespace poi-network

# Install the chart
helm install poi-network ./helm/poi-network \
  --namespace poi-network \
  --set global.environment=staging

# Install with custom values
helm install poi-network ./helm/poi-network \
  --namespace poi-network \
  -f my-values.yaml
```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `global.environment` | Deployment environment | `staging` |
| `global.domainName` | Domain for ingress routes | `poi.network` |
| `global.imageRegistry` | Container image registry | `ghcr.io/quantumworld-dpdns-io/...` |
| `node.replicas` | Number of POI node replicas | `3` |
| `node.resources.requests.cpu` | Node CPU request | `500m` |
| `node.resources.requests.memory` | Node memory request | `512Mi` |
| `node.resources.limits.cpu` | Node CPU limit | `2000m` |
| `node.resources.limits.memory` | Node memory limit | `2Gi` |
| `node.persistence.size` | Persistent volume size | `50Gi` |
| `node.persistence.storageClass` | Storage class for PVCs | `poi-standard` |
| `api.replicas` | API server replicas | `2` |
| `api.autoscaling.enabled` | Enable HPA for API | `true` |
| `api.autoscaling.minReplicas` | Minimum API replicas | `2` |
| `api.autoscaling.maxReplicas` | Maximum API replicas | `10` |
| `dashboard.replicas` | Dashboard replicas | `2` |
| `mcpServer.replicas` | MCP server replicas | `2` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class name | `nginx` |

## Components

- **poi-node**: StatefulSet (3 replicas) - P2P node with vector DB, AI integration, and API
- **poi-api**: Deployment (2 replicas) - REST API server
- **poi-dashboard**: Deployment (2 replicas) - Next.js dashboard
- **poi-mcp-server**: Deployment (2 replicas) - MCP server for AI agent integration

## Networking

| Service | Port | Protocol | Description |
|---------|------|----------|-------------|
| poi-node | 3000 | TCP | HTTP API |
| poi-node | 9090 | TCP | P2P networking |
| poi-node | 9091 | TCP | Prometheus metrics |
| poi-api | 80 | TCP | REST API (ClusterIP) |
| poi-dashboard | 80 | TCP | Dashboard (ClusterIP) |
| poi-mcp-server | 80 | TCP | MCP server (ClusterIP) |

## Storage

- **poi-standard**: gp3 EBS (3000 IOPS, 125 MB/s throughput)
- **poi-fast**: gp3 EBS (16000 IOPS, 1000 MB/s throughput)

## Security

- Pod Security Contexts with non-root users
- Seccomp profiles (RuntimeDefault)
- Capability dropping (ALL)
- Network policies restricting pod communication
- PodDisruptionBudgets for HA

## Upgrading

```bash
helm upgrade poi-network ./helm/poi-network \
  --namespace poi-network \
  --reuse-values \
  --set node.replicas=5
```

## Uninstalling

```bash
helm uninstall poi-network --namespace poi-network
```

## Values Reference

See [values.yaml](values.yaml) for the full configuration tree.
