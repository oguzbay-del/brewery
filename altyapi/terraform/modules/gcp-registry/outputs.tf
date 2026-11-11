output "repository_id" {
  description = "Depo adı."
  value       = google_artifact_registry_repository.this.repository_id
}

output "docker_host" {
  description = "Docker sunucu adı (ör. us-east4-docker.pkg.dev)."
  value       = "${google_artifact_registry_repository.this.location}-docker.pkg.dev"
}

output "image_prefix" {
  description = "İmaj adı öneki: <sunucu>/<proje>/<depo>."
  value       = "${google_artifact_registry_repository.this.location}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.this.repository_id}"
}
