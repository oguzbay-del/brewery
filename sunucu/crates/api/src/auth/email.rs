//! E-mail addresses and the sign-in mail (ADR-0009 rule 2).
//!
//! [`SmtpMailer`] speaks plain SMTP to a relay (mailpit locally; a provider relay later).
//! [`MemoryMailer`] keeps the mails for tests.

use std::{sync::Mutex, time::Duration};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use unicode_normalization::UnicodeNormalization;

use crate::context::BoxFuture;

/// Normalizes an address: trimmed, Unicode NFC, lower case (ADR-0009 rule 2.5). `None` when it
/// is not a plausible address.
#[must_use]
pub fn normalize_email(raw: &str) -> Option<String> {
    let address: String = raw.trim().nfc().collect::<String>().to_lowercase();
    if address.len() > 254 {
        return None;
    }
    let (local, domain) = address.split_once('@')?;
    let bad = |c: char| {
        c.is_control()
            || c.is_whitespace()
            || matches!(
                c,
                '<' | '>' | '(' | ')' | '[' | ']' | '\\' | ',' | ';' | ':' | '"' | '@'
            )
    };
    let local_ok = !local.is_empty() && local.len() <= 64 && !local.chars().any(bad);
    let domain_ok = domain.contains('.')
        && !domain.starts_with(['.', '-'])
        && !domain.ends_with(['.', '-'])
        && !domain.contains("..")
        && !domain.chars().any(bad);
    (local_ok && domain_ok).then_some(address)
}

/// One mail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    /// Recipient (already normalized).
    pub to: String,
    /// Subject (ASCII).
    pub subject: String,
    /// Plain text body.
    pub text: String,
}

/// Why a mail could not be sent.
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    /// The relay could not be reached or broke the conversation.
    #[error("smtp: {0}")]
    Smtp(String),
}

/// Sends mail. Object safe.
pub trait Mailer: Send + Sync + 'static {
    /// Sends `mail`.
    fn send<'a>(&'a self, mail: &'a Mail) -> BoxFuture<'a, Result<(), MailError>>;
}

/// The sign-in mail: the link, what asked for it and the "ignore it" sentence (ADR-0009 rule 2.4).
#[must_use]
pub fn sign_in_mail(to: &str, link: &str, client: &str, country: Option<&str>) -> Mail {
    let place = country.map_or_else(String::new, |c| format!(" near {c}"));
    Mail {
        to: to.to_owned(),
        subject: "Your sign-in link".to_owned(),
        text: format!(
            "Use this link to sign in. It works once and expires in 15 minutes.\n\n{link}\n\n\
             The request came from a {client} device{place}.\n\
             If you did not make this request, ignore this message.\n"
        ),
    }
}

/// Keeps sent mails in memory (tests and development without a relay).
#[derive(Debug, Default)]
pub struct MemoryMailer {
    sent: Mutex<Vec<Mail>>,
}

impl MemoryMailer {
    /// The mails sent so far.
    #[must_use]
    pub fn sent(&self) -> Vec<Mail> {
        self.sent.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl Mailer for MemoryMailer {
    fn send<'a>(&'a self, mail: &'a Mail) -> BoxFuture<'a, Result<(), MailError>> {
        Box::pin(async move {
            if let Ok(mut g) = self.sent.lock() {
                g.push(mail.clone());
            }
            Ok(())
        })
    }
}

/// Plain SMTP client (no TLS, no authentication): for mailpit and trusted in-cluster relays.
#[derive(Debug, Clone)]
pub struct SmtpMailer {
    host: String,
    port: u16,
    from: String,
}

const SMTP_TIMEOUT: Duration = Duration::from_secs(10);

impl SmtpMailer {
    /// A client for the relay at `host:port` sending as `from`.
    #[must_use]
    pub fn new(host: &str, port: u16, from: &str) -> Self {
        Self {
            host: host.to_owned(),
            port,
            from: from.to_owned(),
        }
    }

    async fn converse(&self, mail: &Mail) -> std::io::Result<()> {
        let stream = TcpStream::connect((self.host.as_str(), self.port)).await?;
        let (read, mut write) = stream.into_split();
        let mut read = BufReader::new(read);
        expect(&mut read, 220).await?;
        command(&mut write, &mut read, "EHLO demleme.local", 250).await?;
        command(
            &mut write,
            &mut read,
            &format!("MAIL FROM:<{}>", self.from),
            250,
        )
        .await?;
        command(
            &mut write,
            &mut read,
            &format!("RCPT TO:<{}>", mail.to),
            250,
        )
        .await?;
        command(&mut write, &mut read, "DATA", 354).await?;
        let sent_at = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000");
        let mut data = format!(
            "From: <{}>\r\nTo: <{}>\r\nSubject: {}\r\nDate: {sent_at}\r\nMIME-Version: 1.0\r\n\
             Content-Type: text/plain; charset=us-ascii\r\n\r\n",
            self.from, mail.to, mail.subject
        );
        for line in mail.text.lines() {
            if line.starts_with('.') {
                data.push('.');
            }
            data.push_str(line);
            data.push_str("\r\n");
        }
        data.push_str(".\r\n");
        write.write_all(data.as_bytes()).await?;
        expect(&mut read, 250).await?;
        // The mail is accepted; a failing QUIT does not matter.
        let _ = write.write_all(b"QUIT\r\n").await;
        Ok(())
    }
}

async fn command<R, W>(write: &mut W, read: &mut R, line: &str, ok: u16) -> std::io::Result<()>
where
    R: AsyncBufReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    write.write_all(line.as_bytes()).await?;
    write.write_all(b"\r\n").await?;
    expect(read, ok).await
}

/// Reads one (possibly multi-line) reply and checks its code.
async fn expect<R: AsyncBufReadExt + Unpin>(read: &mut R, ok: u16) -> std::io::Result<()> {
    loop {
        let mut line = String::new();
        if read.read_line(&mut line).await? == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "relay closed the connection",
            ));
        }
        let code: u16 = line.get(..3).and_then(|c| c.parse().ok()).unwrap_or(0);
        if line.as_bytes().get(3) == Some(&b'-') {
            continue; // continuation line
        }
        return if code == ok {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "unexpected reply: {}",
                line.trim_end()
            )))
        };
    }
}

impl Mailer for SmtpMailer {
    fn send<'a>(&'a self, mail: &'a Mail) -> BoxFuture<'a, Result<(), MailError>> {
        Box::pin(async move {
            // Addresses reach the SMTP dialogue verbatim; refuse anything that could add commands.
            if normalize_email(&mail.to).as_deref() != Some(mail.to.as_str())
                || self.from.contains(['\r', '\n', '<', '>'])
                || mail.subject.contains(['\r', '\n'])
            {
                return Err(MailError::Smtp("refused unsafe header value".to_owned()));
            }
            match tokio::time::timeout(SMTP_TIMEOUT, self.converse(mail)).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(MailError::Smtp(e.to_string())),
                Err(_) => Err(MailError::Smtp("timeout".to_owned())),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_normalized_or_refused() {
        assert_eq!(
            normalize_email("  Brew.Er@Example.COM ").as_deref(),
            Some("brew.er@example.com")
        );
        // NFC: "e" + combining acute becomes the single code point.
        assert_eq!(
            normalize_email("cafe\u{301}@example.com").as_deref(),
            Some("caf\u{e9}@example.com")
        );
        for bad in [
            "",
            "plain",
            "a@b",
            "@example.com",
            "a@@example.com",
            "a b@example.com",
            "a@example.com\r\nRCPT TO:<x@y.z>",
            "<a@example.com>",
            "a@.example.com",
            "a@exa..mple.com",
        ] {
            assert_eq!(normalize_email(bad), None, "{bad:?}");
        }
        assert_eq!(
            normalize_email(&format!("{}@example.com", "a".repeat(65))),
            None
        );
    }

    #[test]
    fn mail_says_what_asked_and_how_to_ignore_it() {
        let m = sign_in_mail("a@example.com", "https://x/y?token=t", "iOS", Some("US"));
        assert!(m.text.contains("https://x/y?token=t"));
        assert!(m.text.contains("iOS device near US"));
        assert!(m.text.contains("ignore this message"));
    }
}
