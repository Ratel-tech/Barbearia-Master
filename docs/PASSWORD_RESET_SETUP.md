# Password Reset Setup

## Current Implementation

The system already supports the password reset lifecycle:

1. `POST /api/auth/forgot-password` receives an e-mail.
2. If the account exists, the backend creates a 30-minute reset token.
3. `POST /api/auth/reset-password` receives the token and a new password.
4. The backend updates the password hash, marks the token as used, and invalidates existing sessions for that account.

The flow works for salon admin users and active professionals because both use the same login screen.

## Delivery Still Required

Automatic token delivery is intentionally separated from the password rule. Before production use, configure one delivery channel:

- SMTP/e-mail provider.
- WhatsApp/SMS provider.
- Internal admin-assisted process.

The delivery service should receive the generated token or a frontend URL containing the token and send it to the account owner.

## Local/Development Token Exposure

For local testing only, the backend can expose the generated token in the API response:

```powershell
$env:PASSWORD_RESET_EXPOSE_TOKEN='1'
cargo run
```

Do not enable `PASSWORD_RESET_EXPOSE_TOKEN=1` in production.

## Frontend Entry Points

- `Esqueci minha senha`: requests a recovery code for the e-mail.
- `Já tenho código`: accepts the recovery code and the new password.

## Production Checklist

- Choose and configure the delivery provider.
- Add environment variables for provider credentials.
- Send the reset URL or token after `forgot-password` creates the token.
- Keep the API response generic to avoid revealing whether an e-mail exists.
- Keep token expiration at 30 minutes unless business policy changes.

