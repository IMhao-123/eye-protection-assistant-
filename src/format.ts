export function formatTime(totalSeconds: number) {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function progressValue(remaining: number, total: number) {
  if (total <= 0) return 0;
  return Math.max(0, Math.min(1, remaining / total));
}
