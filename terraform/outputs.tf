output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the EKS cluster"
  value       = module.eks.cluster_security_group_id
}

output "cluster_oidc_issuer_url" {
  description = "OIDC issuer URL for the EKS cluster"
  value       = module.eks.cluster_oidc_issuer_url
}

output "api_url" {
  description = "API server URL"
  value       = "https://api.${var.domain_name}"
}

output "dashboard_url" {
  description = "Dashboard URL"
  value       = "https://dashboard.${var.domain_name}"
}

output "mcp_server_url" {
  description = "MCP server URL"
  value       = "https://mcp.${var.domain_name}"
}

output "vector_db_host" {
  description = "Vector database host"
  value       = var.vector_db_provider == "chroma" ? module.chromadb[0].endpoint : module.qdrant[0].endpoint
}

output "rds_cluster_endpoint" {
  description = "RDS Aurora cluster endpoint"
  value       = module.rds_aurora.cluster_endpoint
}

output "elasticache_endpoint" {
  description = "ElastiCache Redis endpoint"
  value       = module.elasticache.cluster_endpoint
}

output "analytics_bucket_name" {
  description = "S3 bucket for analytics data"
  value       = module.analytics_bucket.bucket_name
}

output "node_role_arn" {
  description = "IAM role ARN for EKS nodes"
  value       = module.eks.eks_managed_node_groups["poi-node-group"].node_group_role_name
}

output "kubeconfig_command" {
  description = "Command to configure kubeconfig"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${module.eks.cluster_name}"
}

output "helm_release_status" {
  description = "Status of the Helm release"
  value       = helm_release.poi_network.status
}
