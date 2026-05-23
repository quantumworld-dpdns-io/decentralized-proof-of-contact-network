terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }

  backend "s3" {
    bucket         = "poi-terraform-state"
    key            = "terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "poi-terraform-locks"
  }
}

# Data sources
data "aws_eks_cluster_auth" "this" {
  name = module.eks.cluster_name
}

data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# VPC
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "${var.cluster_name}-vpc"
  cidr = var.vpc_cidr

  azs             = var.availability_zones
  private_subnets = var.private_subnet_cidrs
  public_subnets  = var.public_subnet_cidrs

  enable_nat_gateway   = true
  single_nat_gateway   = var.environment == "staging" ? true : false
  enable_dns_hostnames = true
  enable_dns_support   = true

  public_subnet_tags = {
    "kubernetes.io/role/elb" = "1"
  }

  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = "1"
  }

  tags = {
    Environment = var.environment
  }
}

# EKS Cluster
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = var.cluster_name
  cluster_version = var.cluster_version

  cluster_endpoint_public_access = var.environment == "production" ? false : true

  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    poi-node-group = {
      name         = var.node_group_name
      min_size     = var.node_min_size
      max_size     = var.node_max_size
      desired_size = var.node_desired_size

      instance_types = var.node_instance_types
      disk_size      = var.node_disk_size

      subnet_ids = module.vpc.private_subnets

      use_name_prefix = false

      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size           = var.node_disk_size
            volume_type           = "gp3"
            iops                  = 3000
            throughput            = 125
            encrypted             = true
            delete_on_termination = true
          }
        }
      }

      tags = {
        Environment = var.environment
        "k8s.io/cluster-autoscaler/${var.cluster_name}" = "owned"
        "k8s.io/cluster-autoscaler/enabled"              = "true"
      }
    }
  }

  node_security_group_additional_rules = {
    ingress_self_all = {
      description = "Node to node all ports"
      protocol    = "-1"
      from_port   = 0
      to_port     = 0
      type        = "self"
      self        = true
    }
    ingress_cluster_all = {
      description = "Cluster to node all ports"
      protocol    = "-1"
      from_port   = 0
      to_port     = 0
      type        = "ingress"
      source_cluster_security_group = true
    }
    egress_all = {
      description = "Node all egress"
      protocol    = "-1"
      from_port   = 0
      to_port     = 0
      type        = "egress"
      cidr_blocks = ["0.0.0.0/0"]
    }
  }

  tags = {
    Environment = var.environment
  }
}

# IAM roles for service accounts
module "iam_assumable_role_poi_node" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-assumable-role-with-oidc"
  version = "~> 5.0"

  create_role = true

  role_name = "poi-node-${var.environment}"

  provider_url = module.eks.cluster_oidc_issuer_url

  role_policy_arns = {
    s3_access = aws_iam_policy.poi_node_s3.arn
  }

  oidc_fully_qualified_subjects = [
    "system:serviceaccount:poi-network:poi-node"
  ]

  tags = {
    Environment = var.environment
  }
}

resource "aws_iam_policy" "poi_node_s3" {
  name        = "poi-node-s3-${var.environment}"
  description = "Allow POI node to access analytics S3 bucket"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          module.analytics_bucket.bucket_arn,
          "${module.analytics_bucket.bucket_arn}/*"
        ]
      },
    ]
  })
}

# RDS Aurora for metadata storage
module "rds_aurora" {
  source  = "terraform-aws-modules/rds-aurora/aws"
  version = "~> 9.0"

  name              = "poi-metadata-${var.environment}"
  engine            = "aurora-postgresql"
  engine_mode       = "provisioned"
  engine_version    = "15.4"
  database_name     = "poinetwork"
  master_username   = "poi_admin"
  master_password   = random_password.rds_master.result
  port              = 5432
  storage_encrypted = true

  vpc_id                = module.vpc.vpc_id
  subnets               = module.vpc.private_subnets
  create_security_group = true
  allowed_cidr_blocks   = module.vpc.private_subnets_cidr_blocks

  instance_class = "db.r6g.large"
  instances = {
    1 = { instance_type = "db.r6g.large" }
    2 = { instance_type = "db.r6g.large" }
  }

  backup_retention_period = var.environment == "production" ? 30 : 7
  preferred_backup_window = "03:00-04:00"

  enabled_cloudwatch_logs_exports = ["postgresql"]

  tags = {
    Environment = var.environment
  }
}

resource "random_password" "rds_master" {
  length  = 24
  special = false
}

# ElastiCache Redis for caching
module "elasticache" {
  source  = "terraform-aws-modules/elasticache/aws"
  version = "~> 3.0"

  cluster_id = "poi-cache-${var.environment}"
  engine     = "redis"

  node_type = var.vector_db_instance_type

  num_cache_nodes = var.environment == "production" ? 2 : 1

  subnet_ids = module.vpc.private_subnets
  vpc_id     = module.vpc.vpc_id

  parameter_group_family = "redis7"
  port                   = 6379

  maintenance_window = "sun:05:00-sun:06:00"

  tags = {
    Environment = var.environment
  }
}

# S3 bucket for analytics
module "analytics_bucket" {
  source  = "terraform-aws-modules/s3-bucket/aws"
  version = "~> 4.0"

  bucket = "poi-analytics-${data.aws_caller_identity.current.account_id}-${var.environment}"

  force_destroy = var.environment == "staging"

  control_object_ownership = true
  object_ownership         = "BucketOwnerPreferred"

  versioning = {
    enabled = var.environment == "production"
  }

  server_side_encryption_configuration = {
    rule = {
      apply_server_side_encryption_by_default = {
        sse_algorithm = "AES256"
      }
    }
  }

  lifecycle_rule = [
    {
      id      = "expire-old-data"
      enabled = true
      expiration = {
        days = var.environment == "production" ? 365 : 90
      }
    }
  ]

  tags = {
    Environment = var.environment
  }
}

# Helm release
resource "helm_release" "poi_network" {
  name       = "poi-network"
  namespace  = "poi-network"
  create_namespace = true

  chart      = var.helm_chart_path
  version    = "0.1.0"

  timeout    = 600
  wait       = true
  wait_for_jobs = true
  lint       = true

  values = [
    yamlencode({
      global = {
        environment = var.environment
        domainName  = var.domain_name
        imageRegistry = "ghcr.io/quantumworld-dpdns-io/decentralized-proof-of-contact-network"
        imagePullPolicy = "IfNotPresent"
      }

      node = {
        replicas = var.poi_node_replicas
        persistence = {
          size = var.environment == "production" ? "100Gi" : "50Gi"
        }
        resources = {
          requests = {
            cpu    = "500m"
            memory = "512Mi"
          }
          limits = {
            cpu    = "2000m"
            memory = "2Gi"
          }
        }
      }

      api = {
        replicas = var.poi_api_replicas
        resources = {
          requests = {
            cpu    = "250m"
            memory = "256Mi"
          }
          limits = {
            cpu    = "1000m"
            memory = "1Gi"
          }
        }
        autoscaling = {
          enabled         = true
          minReplicas     = var.poi_api_replicas
          maxReplicas     = 10
          targetCPU       = 70
          targetMemory    = 80
        }
      }

      dashboard = {
        replicas = var.poi_dashboard_replicas
      }

      mcpServer = {
        replicas = var.poi_mcp_server_replicas
      }

      storage = {
        vectorStoreProvider = var.vector_db_provider
        chromaHost          = "chromadb.poi-network.svc.cluster.local"
        chromaPort          = 8000
        qdrantGrpcPort      = 6333
        qdrantRestPort      = 6334
      }

      ai = {
        provider        = var.ai_provider
        model           = var.ai_model
        embeddingModel  = var.embedding_model
        ollamaEndpoint  = "http://ollama.poi-network.svc.cluster.local:11434"
      }

      ingress = {
        enabled    = true
        className  = "nginx"
        annotations = {
          "cert-manager.io/cluster-issuer" = "letsencrypt-prod"
        }
        tls = [
          {
            hosts      = ["api.${var.domain_name}", "dashboard.${var.domain_name}", "mcp.${var.domain_name}"]
            secretName = "poi-tls"
          }
        ]
      }

      monitoring = {
        prometheusEnabled = var.prometheus_enabled
        grafanaEnabled    = var.grafana_enabled
        otelCollectorEnabled = var.otel_collector_enabled
      }
    })
  ]

  depends_on = [
    module.eks
  ]
}
