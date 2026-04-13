import { isInRanges } from '../../../shared/utils';

export function validatePort(value: number, allowedPortRanges: [number, number][]): boolean {
  return isInRanges(value, allowedPortRanges);
}

export function validatePortString(value: string, allowedPortRanges: [number, number][]): boolean {
  const numericValue = parseInt(value, 10);
  if (Number.isNaN(numericValue)) return false;
  return validatePort(numericValue, allowedPortRanges);
}

export function formatPortRanges(portRanges: [number, number][]): string {
  return portRanges
    .map(([start, end]) => (start === end ? `${start}` : `${start}-${end}`))
    .join(', ');
}
