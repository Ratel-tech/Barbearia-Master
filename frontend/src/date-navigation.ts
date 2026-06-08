export type CalendarDay = {
  date: string;
  day: number;
  currentMonth: boolean;
  today: boolean;
};

export function formatDateKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function addDays(dateKey: string, days: number) {
  const date = parseDateKey(dateKey);
  date.setDate(date.getDate() + days);
  return formatDateKey(date);
}

export function addMonths(dateKey: string, months: number) {
  const date = parseDateKey(dateKey);
  date.setMonth(date.getMonth() + months);
  return formatDateKey(date);
}

export function monthLabel(dateKey: string) {
  return parseDateKey(dateKey).toLocaleDateString("pt-BR", { month: "long", year: "numeric" });
}

export function fullDateLabel(dateKey: string) {
  return parseDateKey(dateKey).toLocaleDateString("pt-BR", {
    weekday: "long",
    day: "2-digit",
    month: "long",
  });
}

export function buildMonthDays(monthDateKey: string, todayKey = formatDateKey(new Date())) {
  const monthDate = parseDateKey(monthDateKey);
  const firstOfMonth = new Date(monthDate.getFullYear(), monthDate.getMonth(), 1);
  const start = new Date(firstOfMonth);
  start.setDate(firstOfMonth.getDate() - firstOfMonth.getDay());

  return Array.from({ length: 42 }, (_, index): CalendarDay => {
    const current = new Date(start);
    current.setDate(start.getDate() + index);
    const date = formatDateKey(current);
    return {
      date,
      day: current.getDate(),
      currentMonth: current.getMonth() === monthDate.getMonth(),
      today: date === todayKey,
    };
  });
}

function parseDateKey(dateKey: string) {
  const [year, month, day] = dateKey.split("-").map(Number);
  return new Date(year, month - 1, day);
}
