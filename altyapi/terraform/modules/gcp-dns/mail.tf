variable "mail_authentication" {
  description = "Provider-approved SPF, DKIM and DMARC TXT values for mail.gristora.com. Supply unquoted TXT values."
  type = object({
    spf   = string
    dkim  = map(string)
    dmarc = string
  })

  validation {
    condition = (
      startswith(var.mail_authentication.spf, "v=spf1 ") &&
      startswith(var.mail_authentication.dmarc, "v=DMARC1;") &&
      length(var.mail_authentication.dkim) > 0 &&
      alltrue([
        for selector, record in var.mail_authentication.dkim :
        can(regex("^[a-zA-Z0-9][a-zA-Z0-9-]*$", selector)) && length(trimspace(record)) > 0
      ])
    )
    error_message = "Supply an SPF policy, a DMARC policy and at least one provider DKIM selector and TXT value."
  }
}

locals {
  mail_dns_name = "mail.${google_dns_managed_zone.gristora.dns_name}"
  mail_txt_records = merge(
    {
      (local.mail_dns_name)            = var.mail_authentication.spf
      ("_dmarc.${local.mail_dns_name}") = var.mail_authentication.dmarc
    },
    {
      for selector, record in var.mail_authentication.dkim :
      "${selector}._domainkey.${local.mail_dns_name}" => record
    }
  )
}

resource "google_dns_record_set" "mail_authentication" {
  for_each = local.mail_txt_records

  project      = var.project_id
  managed_zone = google_dns_managed_zone.gristora.name
  name         = each.key
  type         = "TXT"
  ttl          = 300
  rrdatas      = [join(" ", [for part in regexall(".{1,255}", each.value) : jsonencode(part)])]
}
