terraform {
  required_version = "~> 1.17"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 8.11.0"
    }
  }

  # Durum kovası henüz yok (ürün sahibi hesabı kurunca açılır). Kova açılınca yorumu kaldır;
  # bölge başına ayrı önek kullan (ör. canli/us-east4, canli/europe-west2).
  # backend "gcs" {
  #   bucket = "<DURUM_KOVASI_ADI>"
  #   prefix = "terraform/canli/<BOLGE>"
  # }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

locals {
  environment = "canli"
  region_code = lookup({ "us-east4" = "us", "europe-west2" = "uk" }, var.region)
  name        = "${var.name_prefix}-${local.environment}-${local.region_code}"
  labels = {
    app         = var.name_prefix
    environment = local.environment
    region-code = local.region_code
    managed-by  = "terraform"
  }

  apis = [
    "artifactregistry.googleapis.com",
    "cloudkms.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "compute.googleapis.com",
    "container.googleapis.com",
    "iam.googleapis.com",
    "logging.googleapis.com",
    "monitoring.googleapis.com",
    "redis.googleapis.com",
    "secretmanager.googleapis.com",
    "servicenetworking.googleapis.com",
    "sqladmin.googleapis.com",
    "storage.googleapis.com",
  ]
}

resource "google_project_service" "apis" {
  for_each = toset(local.apis)

  project            = var.project_id
  service            = each.value
  disable_on_destroy = false
}

module "network" {
  source = "../../modules/gcp-network"

  project_id    = var.project_id
  name          = local.name
  region        = var.region
  subnet_cidr   = var.subnet_cidr
  pods_cidr     = var.pods_cidr
  services_cidr = var.services_cidr

  depends_on = [google_project_service.apis]
}

module "gke" {
  source = "../../modules/gcp-gke"

  project_id                 = var.project_id
  name                       = local.name
  region                     = var.region
  network_id                 = module.network.network_id
  subnet_id                  = module.network.subnet_id
  pods_range_name            = module.network.pods_range_name
  services_range_name        = module.network.services_range_name
  master_authorized_networks = var.master_authorized_networks
  release_channel            = var.gke_release_channel
  deletion_protection        = var.deletion_protection
  labels                     = local.labels

  depends_on = [google_project_service.apis]
}

module "sql" {
  source = "../../modules/gcp-sql"

  project_id           = var.project_id
  name                 = "${local.name}-pg"
  region               = var.region
  network_id           = module.network.network_id
  tier                 = var.sql_tier
  availability_type    = var.sql_availability_type
  disk_size_gb         = var.sql_disk_size_gb
  read_replica_count   = var.sql_read_replica_count
  iam_service_accounts = var.sql_iam_service_accounts
  deletion_protection  = var.deletion_protection
  labels               = local.labels

  depends_on = [module.network, google_project_service.apis]
}

module "redis" {
  source = "../../modules/gcp-redis"

  project_id     = var.project_id
  name           = "${local.name}-redis"
  region         = var.region
  network_id     = module.network.network_id
  tier           = var.redis_tier
  memory_size_gb = var.redis_memory_size_gb
  labels         = local.labels

  depends_on = [module.network, google_project_service.apis]
}

module "storage" {
  source = "../../modules/gcp-storage"

  project_id             = var.project_id
  name_prefix            = "${var.project_id}-${local.region_code}"
  location               = var.region
  cors_origins           = var.storage_cors_origins
  audit_retention_days   = var.audit_retention_days
  audit_retention_locked = var.audit_retention_locked
  force_destroy          = var.storage_force_destroy
  labels                 = local.labels

  depends_on = [google_project_service.apis]
}

module "registry" {
  source = "../../modules/gcp-registry"

  project_id    = var.project_id
  repository_id = local.name
  location      = var.region
  reader_members = [
    "serviceAccount:${module.gke.node_service_account_email}",
  ]
  writer_members = var.registry_writer_members
  labels         = local.labels

  depends_on = [google_project_service.apis]
}

module "secrets" {
  source = "../../modules/gcp-secrets"

  project_id = var.project_id
  name       = local.name
  location   = var.region
  secret_ids = var.secret_ids
  labels     = local.labels

  depends_on = [google_project_service.apis]
}
