variable "environment" {
  description = "Deployment environment (staging/production)"
  type        = string
  default     = "staging"

  validation {
    condition     = contains(["staging", "production"], var.environment)
    error_message = "Environment must be 'staging' or 'production'."
  }
}

variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-east-1"
}

variable "cluster_name" {
  description = "EKS cluster name"
  type        = string
  default     = "poi-network-eks"
}

variable "cluster_version" {
  description = "Kubernetes version for EKS"
  type        = string
  default     = "1.30"
}

variable "node_group_name" {
  description = "EKS managed node group name"
  type        = string
  default     = "poi-node-group"
}

variable "node_instance_types" {
  description = "EC2 instance types for node group"
  type        = list(string)
  default     = ["t3.medium", "t3.large"]
}

variable "node_desired_size" {
  description = "Desired number of nodes"
  type        = number
  default     = 3
}

variable "node_min_size" {
  description = "Minimum number of nodes"
  type        = number
  default     = 3
}

variable "node_max_size" {
  description = "Maximum number of nodes"
  type        = number
  default     = 10
}

variable "node_disk_size" {
  description = "Node disk size in GB"
  type        = number
  default     = 100
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "AWS availability zones"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

variable "vector_db_provider" {
  description = "Vector database provider (chroma/qdrant)"
  type        = string
  default     = "chroma"

  validation {
    condition     = contains(["chroma", "qdrant"], var.vector_db_provider)
    error_message = "Vector DB provider must be 'chroma' or 'qdrant'."
  }
}

variable "vector_db_instance_type" {
  description = "RDS/ElastiCache instance type for vector DB"
  type        = string
  default     = "r6g.large"
}

variable "ai_provider" {
  description = "AI provider (ollama/openai)"
  type        = string
  default     = "ollama"

  validation {
    condition     = contains(["ollama", "openai"], var.ai_provider)
    error_message = "AI provider must be 'ollama' or 'openai'."
  }
}

variable "ai_model" {
  description = "AI model name"
  type        = string
  default     = "llama3"
}

variable "embedding_model" {
  description = "Embedding model name"
  type        = string
  default     = "nomic-embed-text"
}

variable "poi_node_replicas" {
  description = "Number of POI node replicas"
  type        = number
  default     = 3
}

variable "poi_api_replicas" {
  description = "Number of POI API replicas"
  type        = number
  default     = 2
}

variable "poi_dashboard_replicas" {
  description = "Number of dashboard replicas"
  type        = number
  default     = 2
}

variable "poi_mcp_server_replicas" {
  description = "Number of MCP server replicas"
  type        = number
  default     = 2
}

variable "domain_name" {
  description = "Domain name for ingress"
  type        = string
  default     = "poi.network"
}

variable "helm_chart_path" {
  description = "Path to the Helm chart"
  type        = string
  default     = "../helm/poi-network"
}

variable "otel_collector_enabled" {
  description = "Enable OpenTelemetry collector"
  type        = bool
  default     = true
}

variable "prometheus_enabled" {
  description = "Enable Prometheus monitoring stack"
  type        = bool
  default     = true
}

variable "grafana_enabled" {
  description = "Enable Grafana dashboards"
  type        = bool
  default     = true
}
