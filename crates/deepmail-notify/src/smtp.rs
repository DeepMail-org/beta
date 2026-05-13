use crate::config::NotifyConfig;
use crate::error::NotifyError;
use crate::pipeline::NotifyEvent;
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::fmt::Write;

pub fn build_smtp_transport(
    cfg: &NotifyConfig,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, NotifyError> {
    if cfg.smtp_host.is_empty() {
        return Err(NotifyError::SmtpNotConfigured);
    }

    let creds = Credentials::new(cfg.smtp_user.clone(), cfg.smtp_password.clone());

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)
        .map_err(|e| NotifyError::SmtpError(format!("SMTP relay config error: {}", e)))?
        .port(cfg.smtp_port)
        .credentials(creds)
        .build();

    Ok(transport)
}

pub async fn send_alert_email(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    to: &str,
    subject: &str,
    html_body: &str,
    from: &str,
) -> Result<(), NotifyError> {
    let from_mbox: Mailbox = from
        .parse()
        .map_err(|e| NotifyError::SmtpError(format!("invalid from address: {}", e)))?;
    let to_mbox: Mailbox = to
        .parse()
        .map_err(|e| NotifyError::SmtpError(format!("invalid to address: {}", e)))?;

    let email = Message::builder()
        .from(from_mbox)
        .to(to_mbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| NotifyError::SmtpError(format!("email build error: {}", e)))?;

    transport
        .send(email)
        .await
        .map_err(|e| NotifyError::SmtpError(format!("SMTP send failed: {}", e)))?;

    Ok(())
}

pub fn render_alert_html(event: &NotifyEvent, dashboard_url: &str) -> String {
    let mut html = String::with_capacity(4096);

    let verdict_color = match event.verdict.as_str() {
        "CLEAN" => "#27ae60",
        "LOW_RISK" => "#2980b9",
        "SUSPICIOUS" => "#e67e22",
        "PHISHING" => "#e74c3c",
        "MALICIOUS" | "MALWARE" => "#8e1200",
        _ => "#7f8c8d",
    };

    let _ = write!(
        html,
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="margin:0;padding:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#f5f5f5;">
<div style="max-width:600px;margin:20px auto;background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.1);">
<div style="background:#1a1a2e;padding:24px;text-align:center;">
<h1 style="margin:0;color:#fff;font-size:22px;">DeepMail Security Alert</h1>
</div>
<div style="padding:24px;">
<div style="text-align:center;margin-bottom:24px;">
<span style="display:inline-block;padding:8px 24px;border-radius:20px;background:{verdict_color};color:#fff;font-weight:bold;font-size:18px;">{verdict}</span>
</div>
<table style="width:100%;border-collapse:collapse;margin-bottom:24px;">
<tr style="border-bottom:1px solid #eee;">
<td style="padding:10px 0;color:#666;font-weight:600;">Email ID</td>
<td style="padding:10px 0;text-align:right;font-family:monospace;font-size:13px;">{email_id}</td>
</tr>
<tr style="border-bottom:1px solid #eee;">
<td style="padding:10px 0;color:#666;font-weight:600;">Event</td>
<td style="padding:10px 0;text-align:right;">{event_type}</td>
</tr>
<tr style="border-bottom:1px solid #eee;">
<td style="padding:10px 0;color:#666;font-weight:600;">Score</td>
<td style="padding:10px 0;text-align:right;font-weight:bold;">{score:.2}</td>
</tr>
<tr>
<td style="padding:10px 0;color:#666;font-weight:600;">Timestamp</td>
<td style="padding:10px 0;text-align:right;">{timestamp}</td>
</tr>
</table>
<div style="text-align:center;margin-top:24px;">
<a href="{dashboard_url}/reports/{email_id}" style="display:inline-block;padding:12px 32px;background:#1a1a2e;color:#fff;text-decoration:none;border-radius:6px;font-weight:600;">View Full Report</a>
</div>
</div>
<div style="background:#f9f9f9;padding:16px 24px;text-align:center;color:#999;font-size:12px;">
DeepMail — Automated Email Threat Analysis
</div>
</div>
</body>
</html>"#,
        verdict_color = verdict_color,
        verdict = escape_html(&event.verdict),
        email_id = event.email_id,
        event_type = escape_html(&event.event_type),
        score = event.score,
        timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        dashboard_url = escape_html(dashboard_url),
    );

    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
