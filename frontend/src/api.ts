import type { AccountType } from "./auth-account";

const API_URL = import.meta.env?.VITE_API_URL ?? "http://127.0.0.1:8080";
const TOKEN_KEY = "stitch_barbershop_token";

export type AuthUser = {
  id: number;
  name: string;
  email: string;
  role: "owner" | "admin" | "barber" | "reception";
  barbershop_id: number;
  barbershop_name: string;
  barber_id?: number;
};

export type AuthResponse = {
  token: string;
  user: AuthUser;
};

export type PasswordResetResponse = {
  message: string;
  reset_token?: string;
};

export type Client = {
  id: number;
  name: string;
  phone: string;
  email?: string;
  document?: string;
  haircut_frequency?: string;
  total_spent_cents: number;
  visits: number;
};

export type Barber = {
  id: number;
  name: string;
  document: string;
  email: string;
  specialty: string;
  status: string;
  monthly_commission_cents: number;
  monthly_tips_cents: number;
  completed_services: number;
};

export type Service = {
  id: number;
  name: string;
  description: string;
  duration_minutes: number;
  price_cents: number;
  category: string;
  active: boolean;
};

export type Appointment = {
  id: number;
  client_id: number;
  client_name: string;
  barber_id: number;
  barber_name: string;
  starts_at: string;
  status: string;
  total_cents: number;
  services: string;
  service_ids: string;
};

export type Commission = {
  barber_id: number;
  service_id: number;
  service_name: string;
  price_cents: number;
  commission_percent: number;
  estimated_return_cents: number;
};

export type Overview = {
  revenue_cents: number;
  commissions_cents: number;
  net_revenue_cents: number;
  extra_expenses_cents: number;
  profit_cents: number;
  clients: number;
  appointments: number;
  open_appointments: number;
  in_progress_appointments: number;
};

export type ExtraExpense = {
  id: number;
  description: string;
  amount_cents: number;
  created_at: string;
};

export type CheckoutPaymentInput = {
  method: string;
  amount_cents: number;
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const token = localStorage.getItem(TOKEN_KEY);
  const response = await fetch(`${API_URL}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...init?.headers,
    },
    ...init,
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(apiErrorMessage(response.status, text));
  }
  return response.json() as Promise<T>;
}

export function apiErrorMessage(status: number, text: string) {
  if (text) {
    try {
      const parsed = JSON.parse(text) as { error?: unknown };
      if (typeof parsed.error === "string") return parsed.error;
    } catch {
      return text;
    }
    return text;
  }
  return `HTTP ${status}`;
}

export const api = {
  hasToken: () => Boolean(localStorage.getItem(TOKEN_KEY)),
  setToken: (token: string) => localStorage.setItem(TOKEN_KEY, token),
  clearToken: () => localStorage.removeItem(TOKEN_KEY),
  login: (body: { email: string; password: string; account_type: AccountType }) =>
    request<AuthResponse>("/api/auth/login", { method: "POST", body: JSON.stringify(body) }),
  registerBarbershop: (body: { barbershop_name: string; owner_name: string; email: string; password: string }) =>
    request<AuthResponse>("/api/auth/register-barbershop", { method: "POST", body: JSON.stringify(body) }),
  forgotPassword: (body: { email: string; account_type: AccountType }) =>
    request<PasswordResetResponse>("/api/auth/forgot-password", { method: "POST", body: JSON.stringify(body) }),
  resetPassword: (body: { token: string; password: string }) =>
    request<PasswordResetResponse>("/api/auth/reset-password", { method: "POST", body: JSON.stringify(body) }),
  me: () => request<AuthUser>("/api/auth/me"),
  overview: () => request<Overview>("/api/overview"),
  clients: () => request<Client[]>("/api/clients"),
  createClient: (body: Partial<Client>) => request<Client>("/api/clients", { method: "POST", body: JSON.stringify(body) }),
  updateClient: (clientId: number, body: Partial<Client>) => request<Client>(`/api/clients/${clientId}`, { method: "PUT", body: JSON.stringify(body) }),
  barbers: () => request<Barber[]>("/api/barbers"),
  createBarber: (body: { name: string; document: string; email: string; password: string; specialty?: string }) =>
    request<Barber>("/api/barbers", { method: "POST", body: JSON.stringify(body) }),
  updateBarber: (barberId: number, body: { name: string; document: string; email: string; password?: string; specialty?: string; status?: string }) =>
    request<Barber>(`/api/barbers/${barberId}`, { method: "PUT", body: JSON.stringify(body) }),
  deleteBarber: (barberId: number) => request<{ deleted: boolean }>(`/api/barbers/${barberId}`, { method: "DELETE" }),
  services: () => request<Service[]>("/api/services"),
  createService: (body: Partial<Service>) => request<Service>("/api/services", { method: "POST", body: JSON.stringify(body) }),
  updateService: (serviceId: number, body: Partial<Service>) => request<Service>(`/api/services/${serviceId}`, { method: "PUT", body: JSON.stringify(body) }),
  appointments: () => request<Appointment[]>("/api/appointments"),
  extraExpenses: () => request<ExtraExpense[]>("/api/extra-expenses"),
  createExtraExpense: (body: { description: string; amount_cents: number }) =>
    request<ExtraExpense>("/api/extra-expenses", { method: "POST", body: JSON.stringify(body) }),
  createAppointment: (body: { client_id: number; barber_id: number; service_ids: number[]; starts_at: string }) =>
    request<Appointment>("/api/appointments", { method: "POST", body: JSON.stringify(body) }),
  updateAppointment: (appointmentId: number, body: { client_id: number; barber_id: number; service_ids: number[]; starts_at: string; status: string }) =>
    request<Appointment>(`/api/appointments/${appointmentId}`, { method: "PUT", body: JSON.stringify(body) }),
  checkout: (body: { appointment_id: number; payment_method: string; paid_cents: number; tip_cents: number; discount_cents: number; payments?: CheckoutPaymentInput[] }) =>
    request<{ change_cents: number; total_cents: number }>("/api/checkouts", { method: "POST", body: JSON.stringify(body) }),
  commissions: (barberId: number) => request<Commission[]>(`/api/barbers/${barberId}/commissions`),
  updateCommission: (barberId: number, body: { service_id: number; commission_percent: number }) =>
    request<Commission[]>(`/api/barbers/${barberId}/commissions`, { method: "PUT", body: JSON.stringify(body) }),
};
