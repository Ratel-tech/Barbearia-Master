use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub document: Option<String>,
    pub haircut_frequency: Option<String>,
    pub total_spent_cents: i64,
    pub visits: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuthIdentity {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
    pub barbershop_id: i64,
    pub barbershop_name: String,
    pub barber_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthIdentity,
}

#[derive(Debug, Deserialize)]
pub struct RegisterBarbershop {
    pub barbershop_name: String,
    pub owner_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub account_type: String,
    pub captcha_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
    pub reset_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClient {
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub document: Option<String>,
    pub haircut_frequency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClient {
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub document: Option<String>,
    pub haircut_frequency: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Barber {
    pub id: i64,
    pub name: String,
    pub document: String,
    pub email: String,
    pub specialty: String,
    pub status: String,
    pub monthly_commission_cents: i64,
    pub monthly_tips_cents: i64,
    pub completed_services: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBarber {
    pub name: String,
    pub email: String,
    pub document: String,
    pub password: String,
    pub specialty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBarber {
    pub name: String,
    pub email: String,
    pub document: String,
    pub password: Option<String>,
    pub specialty: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub duration_minutes: i64,
    pub price_cents: i64,
    pub category: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateService {
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i64,
    pub price_cents: i64,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateService {
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i64,
    pub price_cents: i64,
    pub category: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Appointment {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub barber_id: i64,
    pub barber_name: String,
    pub starts_at: String,
    pub status: String,
    pub total_cents: i64,
    pub services: String,
    pub service_ids: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppointment {
    pub client_id: i64,
    pub barber_id: i64,
    pub service_ids: Vec<i64>,
    pub starts_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppointment {
    pub client_id: i64,
    pub barber_id: i64,
    pub service_ids: Vec<i64>,
    pub starts_at: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutPaymentInput {
    pub method: String,
    pub amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub appointment_id: i64,
    pub payment_method: String,
    pub paid_cents: i64,
    pub tip_cents: i64,
    pub discount_cents: i64,
    #[serde(default)]
    pub payments: Vec<CheckoutPaymentInput>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutPaymentLine {
    pub method: String,
    pub amount_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub appointment_id: i64,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tip_cents: i64,
    pub total_cents: i64,
    pub paid_cents: i64,
    pub change_cents: i64,
    pub payment_method: String,
    pub payments: Vec<CheckoutPaymentLine>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExtraExpense {
    pub id: i64,
    pub description: String,
    pub amount_cents: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateExtraExpense {
    pub description: String,
    pub amount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct CommissionInput {
    pub service_id: i64,
    pub commission_percent: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Commission {
    pub barber_id: i64,
    pub service_id: i64,
    pub service_name: String,
    pub price_cents: i64,
    pub commission_percent: i64,
    pub estimated_return_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct Overview {
    pub revenue_cents: i64,
    pub commissions_cents: i64,
    pub net_revenue_cents: i64,
    pub extra_expenses_cents: i64,
    pub profit_cents: i64,
    pub clients: i64,
    pub appointments: i64,
    pub open_appointments: i64,
    pub in_progress_appointments: i64,
}
