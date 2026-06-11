import type { AccountType } from "./auth-account";

export function passwordResetPayload(email: string, accountType: AccountType, captchaToken?: string): { email: string; account_type: AccountType; captcha_token?: string };
export function passwordResetPayload(token: string, password: string): { token: string; password: string };
export function passwordResetPayload(emailOrToken: string, accountTypeOrPassword: AccountType | string, captchaToken?: string) {
  if (accountTypeOrPassword === "establishment" || accountTypeOrPassword === "professional") {
    return {
      email: emailOrToken.trim().toLowerCase(),
      account_type: accountTypeOrPassword,
      ...(captchaToken ? { captcha_token: captchaToken } : {}),
    };
  }

  return {
    token: emailOrToken.trim(),
    password: accountTypeOrPassword.trim(),
  };
}

export function passwordResetTokenFromSearch(search: string) {
  return new URLSearchParams(search).get("token")?.trim() ?? "";
}
