// Validation mirrors the Android PersonalVpnViewModel.parseFormData logic:
// - WireGuard keys are base64 strings decoding to exactly 32 bytes.
// - Addresses are IP literals (hostnames are not accepted).
// - Endpoints are "<host>:<port>" where <host> is an IP or DNS name and the port is 1..=65535.
//   DNS resolution happens in the daemon at save-time.
// - Allowed IPs must be non-blank (CIDR validation is deferred to the daemon).

const IPV4_REGEX = /^(\d{1,3}\.){3}\d{1,3}$/;
const IPV6_REGEX = /^\[?[0-9a-fA-F:.]+\]?$/;

export type KeyValidationError = 'empty' | 'invalid';

export function validateWireguardKey(value: string): KeyValidationError | undefined {
  if (value.length === 0) return 'empty';
  // `Buffer` is not available in the Electron renderer (nodeIntegration: false),
  // so we use the browser-standard atob/btoa (also present in Node >=16 for tests).
  let decoded: string;
  try {
    decoded = atob(value);
  } catch {
    return 'invalid';
  }
  if (decoded.length !== 32) return 'invalid';
  // Round-trip to reject non-canonical base64.
  if (btoa(decoded) !== value) return 'invalid';
  return undefined;
}

export function validateIp(value: string): 'empty' | 'invalid' | undefined {
  const trimmed = value.trim();
  if (trimmed.length === 0) return 'empty';
  if (IPV4_REGEX.test(trimmed)) {
    const parts = trimmed.split('.').map((p) => parseInt(p, 10));
    if (parts.every((p) => p >= 0 && p <= 255)) return undefined;
    return 'invalid';
  }
  if (IPV6_REGEX.test(trimmed) && trimmed.includes(':')) {
    return undefined;
  }
  return 'invalid';
}

export type EndpointValidationError = 'empty' | 'invalid-address' | 'invalid-port';

export function validateEndpoint(value: string): EndpointValidationError | undefined {
  const trimmed = value.trim();
  if (trimmed.length === 0) return 'empty';
  const separator = trimmed.lastIndexOf(':');
  if (separator === -1) return 'invalid-address';
  const host = trimmed.slice(0, separator).replace(/^\[|\]$/g, '');
  const portStr = trimmed.slice(separator + 1);
  if (host.length === 0 || portStr.length === 0) return 'invalid-address';
  const port = Number(portStr);
  if (!Number.isInteger(port) || port < 1 || port > 65535) return 'invalid-port';
  return undefined;
}

export function validateAllowedIp(value: string): 'empty' | undefined {
  return value.trim().length === 0 ? 'empty' : undefined;
}
