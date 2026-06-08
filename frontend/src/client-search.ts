import { phoneDigits } from "./client-validation.ts";

type SearchableClient = {
  name: string;
  phone?: string;
  document?: string;
};

function onlyDigits(value: string) {
  return phoneDigits(value);
}

function normalizeText(value: string) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .trim();
}

export function clientSearchMatches(client: SearchableClient, query: string) {
  const textQuery = normalizeText(query);
  const digitQuery = onlyDigits(query);
  if (!textQuery && !digitQuery) return true;

  const name = normalizeText(client.name);
  const phone = onlyDigits(client.phone ?? "");
  const document = onlyDigits(client.document ?? "");

  return (
    (!!textQuery && name.includes(textQuery)) ||
    (!!digitQuery && phone.includes(digitQuery)) ||
    (!!digitQuery && document.includes(digitQuery))
  );
}

export function clientDraftFromSearch(query: string) {
  const digits = onlyDigits(query);
  const hasCpfFormatting = /[.-]/.test(query);
  if (digits.length === 11 && hasCpfFormatting) {
    return { phone: "", document: digits };
  }
  const areaCode = Number(digits.slice(0, 2));
  const looksLikePhone = digits.length === 11 && areaCode >= 11 && areaCode <= 99 && digits[2] === "9";
  if (looksLikePhone) {
    return { phone: digits, document: "" };
  }
  if (digits.length === 11) {
    return { phone: "", document: digits };
  }
  return { phone: "", document: "" };
}
