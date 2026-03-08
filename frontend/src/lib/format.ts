const DATE_TIME_FORMATTER = new Intl.DateTimeFormat("en-GB", {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  hour12: false,
  timeZone: "UTC",
});

export function formatUtcTimestamp(value: string): string {
  return DATE_TIME_FORMATTER.format(new Date(value)).replace(",", "");
}
