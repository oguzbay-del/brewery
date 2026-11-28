variable "mail_authentication" {
  description = "Provider-approved SPF, DKIM and DMARC records. Supply unquoted TXT values and fully qualified DKIM record names."
  type = object({
    spf = string
    dkim = map(object({
      type  = string
      value = string
    }))
    dmarc = string
  })
  default  = null
  nullable = true

  validation {
    condition = var.mail_authentication == null ? true : (
      startswith(var.mail_authentication.spf, "v=spf1 ") &&
      startswith(var.mail_authentication.dmarc, "v=DMARC1;") &&
      length(var.mail_authentication.dkim) > 0 &&
      alltrue([
        for name, record in var.mail_authentication.dkim :
        can(regex("^[a-zA-Z0-9][a-zA-Z0-9-]*[.]_domainkey[.](mail[.])?gristora[.]com[.]$", name)) &&
        contains(["TXT", "CNAME"], record.type) &&
        length(trimspace(record.value)) > 0 &&
        (record.type != "CNAME" || endswith(record.value, "."))
      ])
    )
    error_message = "Supply valid SPF and DMARC policies, and provider DKIM TXT or fully qualified CNAME records in the Gristora domain."
  }
}

locals {
  mail_dns_name = "mail.${google_dns_managed_zone.gristora.dns_name}"
  mail_records = var.mail_authentication == null ? {} : merge(
    {
      (local.mail_dns_name) = {
        type  = "TXT"
        value = var.mail_authentication.spf
      }
      ("_dmarc.${local.mail_dns_name}") = {
        type  = "TXT"
        value = var.mail_authentication.dmarc
      }
      ("_dmarc.${google_dns_managed_zone.gristora.dns_name}") = {
        type  = "TXT"
        value = var.mail_authentication.dmarc
      }
    },
    var.mail_authentication.dkim
  )
}

resource "google_dns_record_set" "mail_authentication" {
  for_each = local.mail_records

  project      = var.project_id
  managed_zone = google_dns_managed_zone.gristora.name
  name         = each.key
  type         = each.value.type
  ttl          = 300
  rrdatas      = each.value.type == "TXT" ? [join(" ", [for part in regexall(".{1,255}", each.value.value) : jsonencode(part)])] : [each.value.value]
}
