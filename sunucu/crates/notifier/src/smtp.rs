//! Plain SMTP client (no TLS, no authentication) for mailpit and trusted in-cluster relays.
//! The API has its own copy for sign-in mail (`api::auth::email`).

use std::time::Duration;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

const TIMEOUT: Duration = Duration::from_secs(10);

/// SMTP relay settings.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// Relay host.
    pub host: String,
    /// Relay port.
    pub port: u16,
    /// Sender address.
    pub from: String,
}

/// One mail. Subject and text are ASCII.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    /// Recipient.
    pub to: String,
    /// Subject.
    pub subject: String,
    /// Plain text body.
    pub text: String,
}

fn safe(s: &str) -> bool {
    !s.is_empty() && !s.contains(['\r', '\n', '<', '>', ' ']) && s.contains('@')
}

/// Sends a mail.
///
/// # Errors
/// A text that says what failed.
pub async fn send(cfg: &SmtpConfig, mail: &Mail) -> Result<(), String> {
    if !safe(&mail.to) || !safe(&cfg.from) || mail.subject.contains(['\r', '\n']) {
        return Err("refused unsafe header value".to_owned());
    }
    match tokio::time::timeout(TIMEOUT, converse(cfg, mail)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("smtp: {e}")),
        Err(_) => Err("smtp: timeout".to_owned()),
    }
}

async fn converse(cfg: &SmtpConfig, mail: &Mail) -> std::io::Result<()> {
    let stream = TcpStream::connect((cfg.host.as_str(), cfg.port)).await?;
    let (read, mut write) = stream.into_split();
    let mut read = BufReader::new(read);
    expect(&mut read, 220).await?;
    command(&mut write, &mut read, "EHLO demleme.local", 250).await?;
    command(
        &mut write,
        &mut read,
        &format!("MAIL FROM:<{}>", cfg.from),
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
        cfg.from, mail.to, mail.subject
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
    let _ = write.write_all(b"QUIT\r\n").await;
    Ok(())
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
            continue;
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
