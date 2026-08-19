import type { TrendModel } from "./report-view";

export interface PieSlice {
  kind: "rank" | "remainder";
  value: number;
}

function escapeAttr(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

function n(value: number): string {
  return value.toFixed(2);
}

export function reportPieSvg(slices: PieSlice[], ariaLabel: string): string {
  const drawable = slices.filter((slice) => slice.value > 0);
  const sum = drawable.reduce((total, slice) => total + slice.value, 0);
  if (sum <= 0 || drawable.length === 0) {
    return "";
  }
  const size = 160;
  const cx = 80;
  const cy = 80;
  const r = 72;
  const label = escapeAttr(ariaLabel);
  if (drawable.length === 1) {
    const slice = drawable[0];
    const klass = slice?.kind === "remainder" ? "pie-remainder" : "pie-slice-0";
    return `<svg class="report-pie-svg" viewBox="0 0 ${size} ${size}" role="img" aria-label="${label}"><circle class="${klass}" cx="${cx}" cy="${cy}" r="${r}" /></svg>`;
  }
  let angle = -Math.PI / 2;
  const paths: string[] = [];
  let rankIndex = 0;
  for (const slice of drawable) {
    const sweep = (slice.value / sum) * Math.PI * 2;
    const start = angle;
    const end = angle + sweep;
    const x1 = cx + r * Math.cos(start);
    const y1 = cy + r * Math.sin(start);
    const x2 = cx + r * Math.cos(end);
    const y2 = cy + r * Math.sin(end);
    const large = sweep > Math.PI ? 1 : 0;
    const klass = slice.kind === "remainder" ? "pie-remainder" : `pie-slice-${rankIndex % 6}`;
    paths.push(
      `<path class="${klass}" d="M ${n(cx)} ${n(cy)} L ${n(x1)} ${n(y1)} A ${r} ${r} 0 ${large} 1 ${n(x2)} ${n(y2)} Z" />`
    );
    if (slice.kind === "rank") {
      rankIndex += 1;
    }
    angle = end;
  }
  return `<svg class="report-pie-svg" viewBox="0 0 ${size} ${size}" role="img" aria-label="${label}">${paths.join("")}</svg>`;
}

export function reportTrendSvg(model: TrendModel, ariaLabel: string): string {
  if (model.kind === "empty" || model.points.length === 0) {
    return "";
  }
  const width = 320;
  const height = 128;
  const pad = 8;
  const plotW = width - pad * 2;
  const plotH = height - pad * 2;
  const xOf = (unit: number): number => pad + unit * plotW;
  const yOf = (unit: number): number => pad + plotH - unit * plotH;
  const label = escapeAttr(ariaLabel);
  const base = `<line class="trend-base" x1="${pad}" y1="${n(pad + plotH)}" x2="${width - pad}" y2="${n(pad + plotH)}" />`;
  if (model.kind === "single") {
    const point = model.points[0];
    if (!point) {
      return "";
    }
    const barW = 28;
    const gap = 12;
    const mid = width / 2;
    const upH = point.yUp * plotH;
    const downH = point.yDown * plotH;
    return `<svg class="report-trend-svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="${label}">${base}<rect class="trend-up" x="${n(mid - gap / 2 - barW)}" y="${n(pad + plotH - upH)}" width="${barW}" height="${n(upH)}" /><rect class="trend-down" x="${n(mid + gap / 2)}" y="${n(pad + plotH - downH)}" width="${barW}" height="${n(downH)}" /></svg>`;
  }
  const up = model.points.map((point) => `${n(xOf(point.x))},${n(yOf(point.yUp))}`).join(" ");
  const down = model.points.map((point) => `${n(xOf(point.x))},${n(yOf(point.yDown))}`).join(" ");
  return `<svg class="report-trend-svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="${label}">${base}<polyline class="trend-up" fill="none" points="${up}" /><polyline class="trend-down" fill="none" points="${down}" /></svg>`;
}
