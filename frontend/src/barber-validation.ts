import { isValidCpf } from "./client-validation.ts";

export function validateBarberRequiredFields(input: {
  name: string;
  email: string;
  document: string;
  password: string;
  isEditing: boolean;
}) {
  if (input.name.trim().length < 3) return "Informe o nome do profissional.";
  if (!input.email.includes("@") || input.email.trim().length < 6) return "Informe o e-mail do profissional.";
  if (!input.document.trim()) return "Informe o CPF do profissional.";
  if (!isValidCpf(input.document)) return "Informe um CPF válido para o profissional.";
  if (!input.isEditing && input.password.trim().length < 6) return "Informe a senha do profissional.";
  return "";
}
