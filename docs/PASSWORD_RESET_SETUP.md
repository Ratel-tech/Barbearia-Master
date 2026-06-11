# Password Reset Setup

## Current Implementation

The system already supports the password reset lifecycle:

1. `POST /api/auth/forgot-password` receives an e-mail and `account_type`.
2. If the account exists, the backend creates a 30-minute reset token.
3. `POST /api/auth/reset-password` receives the token and a new password.
4. The backend updates the password hash, marks the token as used, and invalidates existing sessions for that account.

The flow works for salon admin users and active professionals. The selected `account_type` decides which table is checked:

- `establishment`: checks `users`.
- `professional`: checks active `barbers`.

## Delivery Configuration

Automatic token delivery supports a local log mode and production SMTP mode.

Local development:

```powershell
$env:PASSWORD_RESET_DELIVERY='log'
cargo run
```

Production e-mail delivery:

```powershell
$env:PASSWORD_RESET_DELIVERY='smtp'
$env:PASSWORD_RESET_PUBLIC_URL='https://app.example.com'
$env:SMTP_HOST='smtp.example.com'
$env:SMTP_PORT='587'
$env:SMTP_USERNAME='mailer@example.com'
$env:SMTP_PASSWORD='provider-password'
$env:SMTP_FROM='no-reply@example.com'
$env:SMTP_FROM_NAME='Barbearia Mestre'
cargo run
```

When SMTP is enabled, the backend validates the SMTP settings before checking whether the requested e-mail exists. Delivery failures are logged and the public API response stays generic to avoid account enumeration.

WhatsApp/SMS delivery is still a future channel; SMTP is the production-ready channel currently implemented.

## Local/Development Token Exposure

For local testing only, the backend can expose the generated token in the API response:

```powershell
$env:PASSWORD_RESET_EXPOSE_TOKEN='1'
cargo run
```

Do not enable `PASSWORD_RESET_EXPOSE_TOKEN=1` in production.

## Frontend Entry Points

- Access selector: must match the account owner, `Estabelecimento` or `Profissional`.
- `Esqueci minha senha`: requests a recovery code for the e-mail and selected access type.
- `Já tenho código`: accepts the recovery code and the new password.
- Recovery links with `?token=...` open the reset form with the recovery code already filled in.

## Production Checklist

- Choose and configure the delivery provider.
- Add environment variables for provider credentials.
- Use `PASSWORD_RESET_DELIVERY=smtp` outside local development.
- Keep the API response generic to avoid revealing whether an e-mail exists.
- Keep token expiration at 30 minutes unless business policy changes.

