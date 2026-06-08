const OPENING_HOUR = 9;
const CLOSING_HOUR = 22;

export function businessHours() {
  return Array.from(
    { length: CLOSING_HOUR - OPENING_HOUR + 1 },
    (_, index) => `${String(OPENING_HOUR + index).padStart(2, "0")}:00`,
  );
}
