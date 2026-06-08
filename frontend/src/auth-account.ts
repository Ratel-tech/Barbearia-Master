export type AccountType = "establishment" | "professional";

export function accountLabel(accountType: AccountType) {
  return accountType === "establishment" ? "Estabelecimento" : "Profissional";
}

export function loginPayload(email: string, password: string, accountType: AccountType) {
  return {
    email: email.trim().toLowerCase(),
    password: password.trim(),
    account_type: accountType,
  };
}
