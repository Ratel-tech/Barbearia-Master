export function appointmentStatusClass(status: string) {
  return {
    scheduled: "is-waiting",
    waiting: "is-waiting",
    in_chair: "is-active",
    completed: "is-completed",
    cancelled: "is-cancelled",
  }[status] ?? "is-waiting";
}

export function canCheckoutAppointment(status: string) {
  return status === "scheduled" || status === "waiting" || status === "in_chair";
}

export function canEditAppointment(status: string) {
  return status !== "completed";
}
