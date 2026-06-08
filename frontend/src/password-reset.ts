export function passwordResetPayload(email: string): { email: string };
export function passwordResetPayload(token: string, password: string): { token: string; password: string };
export function passwordResetPayload(emailOrToken: string, password?: string) {
  if (password === undefined) {
    return { email: emailOrToken.trim().toLowerCase() };
  }

  return {
    token: emailOrToken.trim(),
    password: password.trim(),
  };
}
