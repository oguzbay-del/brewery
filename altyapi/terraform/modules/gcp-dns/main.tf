terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
    }
  }
}

variable "project_id" {
  type        = string
  description = "Google Cloud project that owns the public DNS zone."
}

variable "host_ipv4_addresses" {
  description = "IPv4 addresses for the approved Gristora hosts. Supply real addresses before deployment."
  type = object({
    app  = list(string)
    api  = list(string)
    auth = list(string)
    www  = list(string)
  })

  validation {
    condition = alltrue([
      for addresses in values(var.host_ipv4_addresses) :
      length(addresses) > 0 && alltrue([
        for address in addresses : can(cidrnetmask("${address}/32"))
      ])
    ])
    error_message = "Each host must have at least one valid IPv4 address."
  }
}

resource "google_project_service" "dns" {
  project            = var.project_id
  service            = "dns.googleapis.com"
  disable_on_destroy = false
}

resource "google_dns_managed_zone" "gristora" {
  project     = var.project_id
  name        = "gristora-com"
  dns_name    = "gristora.com."
  description = "Public DNS zone for GRISTORA."
  visibility  = "public"

  depends_on = [google_project_service.dns]
}

resource "google_dns_record_set" "hosts" {
  for_each = var.host_ipv4_addresses

  project      = var.project_id
  managed_zone = google_dns_managed_zone.gristora.name
  name         = "${each.key}.${google_dns_managed_zone.gristora.dns_name}"
  type         = "A"
  ttl          = 300
  rrdatas      = each.value
}

output "name_servers" {
  description = "Name servers for owner review. Do not change delegation before approval."
  value       = google_dns_managed_zone.gristora.name_servers
}
