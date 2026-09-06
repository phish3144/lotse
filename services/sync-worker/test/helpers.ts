import { SELF } from "cloudflare:test";
import { bytesToBase64, randomBytes } from "../src/crypto";

export const BASE = "https://sync-worker.test";

export function randomB64(bytes = 32): string {
  return bytesToBase64(randomBytes(bytes));
}

export function api(path: string, init?: RequestInit & { token?: string }): Promise<Response> {
  const headers = new Headers(init?.headers);
  if (init?.token) headers.set("authorization", `Bearer ${init.token}`);
  if (init?.body && !headers.has("content-type")) headers.set("content-type", "application/json");
  return SELF.fetch(`${BASE}${path}`, { ...init, headers });
}

export function registerBody(overrides: Partial<Record<string, unknown>> = {}) {
  return {
    email: `user-${crypto.randomUUID()}@example.com`,
    auth_key: randomB64(),
    recovery_auth_key: randomB64(),
    salt: randomB64(16),
    kdf: { m: 65536, t: 3, p: 1 },
    wrapped_account_key: randomB64(48),
    wrapped_account_key_recovery: randomB64(48),
    device: { id: crypto.randomUUID(), name: "Test Device", platform: "desktop" },
    ...overrides,
  };
}

export async function registerAccount(overrides: Partial<Record<string, unknown>> = {}) {
  const body = registerBody(overrides);
  const res = await api("/v1/auth/register", { method: "POST", body: JSON.stringify(body) });
  if (res.status !== 201) {
    throw new Error(`register failed: ${res.status} ${await res.text()}`);
  }
  const json = (await res.json()) as { account_id: string; session_token: string };
  return { ...json, requestBody: body, deviceId: (body.device as { id: string }).id };
}

/** Builds a well-formed, fixed-length HLC string for the given millis/counter/device. */
export function hlc(millis: number, counter: number, deviceId: string): string {
  const ts = String(millis).padStart(13, "0");
  const ctr = String(counter).padStart(4, "0");
  // Pad/trim device id into the 26-char Crockford-base32-looking suffix the
  // server's HLC_PATTERN expects; tests don't need it to be a "real" ULID.
  const suffix = (deviceId.replace(/[^0-9A-HJKMNP-TV-Z]/gi, "").toUpperCase() + "0".repeat(26)).slice(0, 26);
  return `${ts}-${ctr}-${suffix}`;
}

export function makeEnvelope(overrides: Partial<Record<string, unknown>> = {}) {
  return {
    id: crypto.randomUUID(),
    kind: "note",
    hlc: hlc(Date.now(), 1, "DEVICE"),
    device_id: "DEVICE",
    deleted: false,
    format_version: 1,
    nonce: randomB64(24),
    ciphertext: randomB64(64),
    ...overrides,
  };
}
