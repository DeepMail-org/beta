use std::sync::Arc;

use lettre::message::Mailbox;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use tonic::{Request, Response, Status};

use deepmail_common::proto::otp_smtp::{
    otp_smtp_service_server::OtpSmtpService, SendOtpRequest, SendOtpResponse,
};

use crate::config::Config;
use crate::error::OtpSmtpError;

pub struct OtpSmtpServiceImpl {
    cfg: Arc<Config>,
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl OtpSmtpServiceImpl {
    pub fn new(cfg: Arc<Config>) -> Result<Self, OtpSmtpError> {
        if !cfg.smtp_enabled {
            return Ok(Self { cfg, mailer: None });
        }

        if cfg.smtp_username.trim().is_empty() || cfg.smtp_password.trim().is_empty() {
            return Err(OtpSmtpError::MissingAuth);
        }

        let creds = Credentials::new(cfg.smtp_username.clone(), cfg.smtp_password.clone());
        let builder = if cfg.smtp_starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)
                .map_err(|e| OtpSmtpError::Transport(e.to_string()))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
                .map_err(|e| OtpSmtpError::Transport(e.to_string()))?
        };

        let mailer = builder.credentials(creds).port(cfg.smtp_port).build();
        Ok(Self {
            cfg,
            mailer: Some(mailer),
        })
    }
}

#[tonic::async_trait]
impl OtpSmtpService for OtpSmtpServiceImpl {
    async fn send_otp(
        &self,
        request: Request<SendOtpRequest>,
    ) -> Result<Response<SendOtpResponse>, Status> {
        let req = request.into_inner();

        if req.recipient_email.trim().is_empty() || req.otp_code.trim().is_empty() {
            return Err(Status::invalid_argument(
                "recipient_email and otp_code are required",
            ));
        }

        if !self.cfg.smtp_enabled {
            tracing::warn!(
                recipient = %req.recipient_email,
                "smtp disabled; accepting OTP request without delivery"
            );
            return Ok(Response::new(SendOtpResponse {
                accepted: true,
                message_id: format!("smtp-disabled:{}", uuid::Uuid::new_v4()),
                error: String::new(),
            }));
        }

        let from = Mailbox::new(
            Some(self.cfg.smtp_from_name.clone()),
            self.cfg
                .smtp_from_email
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    Status::from(OtpSmtpError::Address(e.to_string()))
                })?,
        );

        let to = Mailbox::new(
            if req.recipient_name.trim().is_empty() {
                None
            } else {
                Some(req.recipient_name.clone())
            },
            req.recipient_email
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    Status::from(OtpSmtpError::Address(e.to_string()))
                })?,
        );

        let ttl = if req.expires_in_seconds <= 0 {
            600
        } else {
            req.expires_in_seconds
        };

        let body = format!(
            "Your DeepMail OTP code is: {}\n\nThis code expires in {} seconds.\nIf you did not request this, ignore this email.",
            req.otp_code, ttl
        );

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Your DeepMail OTP Code")
            .body(body)
            .map_err(|e| Status::invalid_argument(format!("invalid email message: {e}")))?;

        let Some(mailer) = &self.mailer else {
            return Err(Status::internal("mailer not initialized"));
        };

        match mailer.send(email).await {
            Ok(_) => Ok(Response::new(SendOtpResponse {
                accepted: true,
                message_id: uuid::Uuid::new_v4().to_string(),
                error: String::new(),
            })),
            Err(e) => {
                tracing::error!(error = %e, recipient = %req.recipient_email, "failed to send otp email");
                Ok(Response::new(SendOtpResponse {
                    accepted: false,
                    message_id: String::new(),
                    error: e.to_string(),
                }))
            }
        }
    }
}
