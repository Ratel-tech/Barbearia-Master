export function phoneDigits(value: string) {
  const digits = value.replace(/\D/g, "");
  if (isRepeatedDigitPlaceholder(digits)) return digits.slice(0, 12);
  return digits.slice(0, 11);
}

export function formatMobilePhoneInput(value: string) {
  const digits = phoneDigits(value);
  if (isRepeatedDigitPlaceholder(digits) && digits.length > 11) {
    return `(${digits.slice(0, 3)}) ${digits.slice(3, 8)}-${digits.slice(8)}`;
  }
  if (digits.length <= 2) return digits ? `(${digits}` : "";
  if (digits.length <= 7) return `(${digits.slice(0, 2)}) ${digits.slice(2)}`;
  return `(${digits.slice(0, 2)}) ${digits.slice(2, 7)}-${digits.slice(7)}`;
}

export function isBrazilianMobilePhone(value: string) {
  const digits = phoneDigits(value);
  if (isRepeatedDigitPlaceholder(digits)) return true;
  const areaCode = Number(digits.slice(0, 2));
  return digits.length === 11 && areaCode >= 11 && areaCode <= 99 && digits[2] === "9";
}

function isRepeatedDigitPlaceholder(digits: string) {
  return digits.length >= 3 && digits.split("").every((digit) => digit === digits[0]);
}

export function cpfDigits(value: string) {
  return value.replace(/\D/g, "").slice(0, 11);
}

export function isValidCpf(value: string) {
  const digits = cpfDigits(value);
  if (digits.length !== 11 || isRepeatedDigitPlaceholder(digits)) return false;

  const firstDigit = cpfCheckDigit(digits, 9);
  const secondDigit = cpfCheckDigit(digits, 10);
  return Number(digits[9]) === firstDigit && Number(digits[10]) === secondDigit;
}

function cpfCheckDigit(digits: string, length: number) {
  const sum = digits
    .slice(0, length)
    .split("")
    .reduce((total, digit, index) => total + Number(digit) * (length + 1 - index), 0);
  const remainder = sum % 11;
  return remainder < 2 ? 0 : 11 - remainder;
}

export function validateClientRequiredFields(input: { name: string; phone: string; document?: string; haircut_frequency: string }) {
  if (input.name.trim().length < 3) return "Informe o nome completo do cliente.";
  if (!isBrazilianMobilePhone(input.phone)) return "Informe um celular válido com DDD e 9 dígitos ou uma sequência repetida.";
  if (input.document?.trim() && !isValidCpf(input.document)) return "Informe um CPF válido.";
  if (!input.haircut_frequency.trim()) return "Informe de quanto em quanto tempo o cliente corta o cabelo.";
  return "";
}
