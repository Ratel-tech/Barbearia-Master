import type { AccountType } from "./auth-account";

export function passwordResetPayload(email: string, accountType: AccountType): { email: string; account_type: AccountType };
export function passwordResetPayload(token: string, password: string): { token: string; password: string };
export function passwordResetPayload(emailOrToken: string, accountTypeOrPassword: AccountType | string) {
  if (accountTypeOrPassword === "establishment" || accountTypeOrPassword === "professional") {
    return {
      email: emailOrToken.trim().toLowerCase(),
      account_type: accountTypeOrPassword,
    };
  }

  return {
    token: emailOrToken.trim(),
    password: accountTypeOrPassword.trim(),
  };
}
