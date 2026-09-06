/**
 * Shared types for the Lotse sync worker.
 *
 * The service is intentionally "dumb": it never sees plaintext. Every type
 * here mirrors docs/SYNC_PROTOCOL.md and docs/THREAT_MODEL.md section 3 --
 * only ciphertext, IDs, HLC timestamps, sizes and device names ever live in
 * these shapes.
 */

export interface Env {
  DB: D1Database;
  BLOBS: R2Bucket;
}

/** Device platforms recognized by the protocol (SYNC_PROTOCOL.md section 4/1). */
export type Platform = "desktop" | "web" | "cli";

export function isPlatform(value: unknown): value is Platform {
  return value === "desktop" || value === "web" || value === "cli";
}

export interface DeviceInput {
  id: string;
  name: string;
  platform: Platform;
}

export interface KdfParams {
  m: number;
  t: number;
  p: number;
}

/** The record kinds a Datensatz-Umschlag may carry (SYNC_PROTOCOL.md section 2). */
export const RECORD_KINDS = [
  "project",
  "note",
  "reference",
  "vault_entry",
  "device",
  "settings",
] as const;
export type RecordKind = (typeof RECORD_KINDS)[number];

export function isRecordKind(value: unknown): value is RecordKind {
  return typeof value === "string" && (RECORD_KINDS as readonly string[]).includes(value);
}

/** Well-formed HLC: `<unix_ms:013>-<counter:04>-<device_id ULID>`. */
export const HLC_PATTERN = /^\d{13}-\d{4}-[0-9A-HJKMNP-TV-Z]{26}$/;

/** A Datensatz-Umschlag as sent by clients (server_seq is server-assigned only). */
export interface InboundEnvelope {
  id: string;
  kind: RecordKind;
  hlc: string;
  device_id: string;
  deleted: boolean;
  format_version: number;
  nonce: string; // base64, 24 raw bytes
  ciphertext: string; // base64, <= 256 KiB raw
}

/** A Datensatz-Umschlag as returned to clients, with server_seq set. */
export interface OutboundEnvelope extends InboundEnvelope {
  server_seq: number;
}

/** Row shape of the `records` table. */
export interface RecordRow {
  account_id: string;
  id: string;
  kind: string;
  hlc: string;
  device_id: string;
  deleted: number;
  format_version: number;
  nonce: string;
  ciphertext: string;
  server_seq: number;
}

/** Row shape of the `accounts` table. */
export interface AccountRow {
  id: string;
  email: string;
  auth_hash: string;
  recovery_auth_hash: string;
  salt: string;
  kdf_m: number;
  kdf_t: number;
  kdf_p: number;
  wrapped_account_key: string;
  wrapped_account_key_recovery: string;
  plan: string;
  flags: string;
  created_at: number;
}

/** Row shape of the `devices` table. */
export interface DeviceRow {
  id: string;
  account_id: string;
  name: string;
  platform: string;
  created_at: number;
  last_seen_at: number;
}

/** Established identity of an authenticated request. */
export interface AuthContext {
  accountId: string;
  deviceId: string;
  tokenHash: string;
}

/** Everything a route handler needs. */
export interface RequestContext {
  request: Request;
  env: Env;
  ctx: ExecutionContext;
  params: Record<string, string>;
  auth?: AuthContext;
}
