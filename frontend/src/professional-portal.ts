import type { Appointment, Commission } from "./api";

export function professionalAgendaSummary(appointments: Appointment[], today: string) {
  const todayAppointments = appointments
    .filter((appointment) => appointment.starts_at.slice(0, 10) === today)
    .sort((left, right) => left.starts_at.localeCompare(right.starts_at));
  return {
    today: todayAppointments,
    openCount: todayAppointments.filter((appointment) => appointment.status !== "completed" && appointment.status !== "cancelled").length,
    completedCount: todayAppointments.filter((appointment) => appointment.status === "completed").length,
    nextAppointment: todayAppointments.find((appointment) => appointment.status === "scheduled" || appointment.status === "in_chair") ?? null,
  };
}

export function commissionSummary(commissions: Commission[]) {
  const estimatedReturn = commissions.reduce((total, commission) => total + commission.estimated_return_cents, 0);
  const percentTotal = commissions.reduce((total, commission) => total + commission.commission_percent, 0);
  return {
    services: commissions.length,
    estimated_return_cents: estimatedReturn,
    average_percent: commissions.length === 0 ? 0 : Math.round(percentTotal / commissions.length),
  };
}
