/**
 * All D1 access in one place. Keeps SQL out of the handlers and makes the
 * "one push = one batch" and "throttle last_seen_at writes" rules easy to
 * audit against SYNC_PROTOCOL.md / the task spec.
 */
import type { AccountRow, DeviceRow, Platform, RecordRow } from "./types";

export const LAST_SEEN_THROTTLE_MS = 60_000;

export function sessionDurationMs(platform: Platform): number {
  return platform === "web" ? 12 * 60 * 60 * 1000 : 30 * 24 * 60 * 60 * 1000;
}

export async function getAccountByEmail(db: D1Database, email: string): Promise<AccountRow | null> {
  const row = await db.prepare("SELECT * FROM accounts WHERE email = ?").bind(email).first<AccountRow>();
  return row ?? null;
}

export async function getAccountById(db: D1Database, id: string): Promise<AccountRow | null> {
  const row = await db.prepare("SELECT * FROM accounts WHERE id = ?").bind(id).first<AccountRow>();
  return row ?? null;
}

export function insertAccountStmt(
  db: D1Database,
  account: {
    id: string;
    email: string;
    authHash: string;
    recoveryAuthHash: string;
    salt: string;
    kdfM: number;
    kdfT: number;
    kdfP: number;
    wrappedAccountKey: string;
    wrappedAccountKeyRecovery: string;
    createdAt: number;
  },
): D1PreparedStatement {
  return db
    .prepare(
      `INSERT INTO accounts
        (id, email, auth_hash, recovery_auth_hash, salt, kdf_m, kdf_t, kdf_p,
         wrapped_account_key, wrapped_account_key_recovery, plan, flags, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'free', '{}', ?)`,
    )
    .bind(
      account.id,
      account.email,
      account.authHash,
      account.recoveryAuthHash,
      account.salt,
      account.kdfM,
      account.kdfT,
      account.kdfP,
      account.wrappedAccountKey,
      account.wrappedAccountKeyRecovery,
      account.createdAt,
    );
}

export function upsertDeviceStmt(
  db: D1Database,
  device: { id: string; accountId: string; name: string; platform: Platform; now: number },
): D1PreparedStatement {
  return db
    .prepare(
      `INSERT INTO devices (id, account_id, name, platform, created_at, last_seen_at)
       VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(id) DO UPDATE SET
         name = excluded.name,
         platform = excluded.platform,
         last_seen_at = excluded.last_seen_at`,
    )
    .bind(device.id, device.accountId, device.name, device.platform, device.now, device.now);
}

export function insertSeqRowStmt(db: D1Database, accountId: string): D1PreparedStatement {
  return db.prepare("INSERT INTO seq (account_id, value) VALUES (?, 0)").bind(accountId);
}

export function insertSessionStmt(
  db: D1Database,
  session: { tokenHash: string; accountId: string; deviceId: string; now: number; expiresAt: number },
): D1PreparedStatement {
  return db
    .prepare(
      "INSERT INTO sessions (token_hash, account_id, device_id, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(session.tokenHash, session.accountId, session.deviceId, session.now, session.expiresAt);
}

export interface SessionRow {
  token_hash: string;
  account_id: string;
  device_id: string;
  created_at: number;
  expires_at: number;
}

export async function getSessionByTokenHash(db: D1Database, tokenHash: string): Promise<SessionRow | null> {
  const row = await db.prepare("SELECT * FROM sessions WHERE token_hash = ?").bind(tokenHash).first<SessionRow>();
  return row ?? null;
}

export async function deleteSessionByTokenHash(db: D1Database, tokenHash: string): Promise<void> {
  await db.prepare("DELETE FROM sessions WHERE token_hash = ?").bind(tokenHash).run();
}

export async function deleteOtherSessions(db: D1Database, accountId: string, keepTokenHash: string): Promise<void> {
  await db
    .prepare("DELETE FROM sessions WHERE account_id = ? AND token_hash != ?")
    .bind(accountId, keepTokenHash)
    .run();
}

/** Nach einer Wiederherstellung gilt keine bestehende Sitzung mehr. */
export async function deleteAllSessions(db: D1Database, accountId: string): Promise<void> {
  await db.prepare("DELETE FROM sessions WHERE account_id = ?").bind(accountId).run();
}

export async function deleteSessionsForDevice(db: D1Database, accountId: string, deviceId: string): Promise<void> {
  await db
    .prepare("DELETE FROM sessions WHERE account_id = ? AND device_id = ?")
    .bind(accountId, deviceId)
    .run();
}

/**
 * Updates last_seen_at, but only if it hasn't been touched in the last
 * LAST_SEEN_THROTTLE_MS -- the WHERE guard means the UPDATE affects 0 rows
 * (no D1 write billed) when we're within the throttle window.
 */
export async function touchDeviceLastSeen(
  db: D1Database,
  accountId: string,
  deviceId: string,
  now: number,
): Promise<void> {
  await db
    .prepare(
      "UPDATE devices SET last_seen_at = ? WHERE id = ? AND account_id = ? AND last_seen_at < ?",
    )
    .bind(now, deviceId, accountId, now - LAST_SEEN_THROTTLE_MS)
    .run();
}

export async function listDevices(db: D1Database, accountId: string): Promise<DeviceRow[]> {
  const { results } = await db
    .prepare("SELECT * FROM devices WHERE account_id = ? ORDER BY created_at ASC")
    .bind(accountId)
    .all<DeviceRow>();
  return results;
}

export async function getDevice(db: D1Database, accountId: string, deviceId: string): Promise<DeviceRow | null> {
  const row = await db
    .prepare("SELECT * FROM devices WHERE id = ? AND account_id = ?")
    .bind(deviceId, accountId)
    .first<DeviceRow>();
  return row ?? null;
}

export async function deleteDevice(db: D1Database, accountId: string, deviceId: string): Promise<void> {
  await db.prepare("DELETE FROM devices WHERE id = ? AND account_id = ?").bind(deviceId, accountId).run();
}

export async function updateAccountPassword(
  db: D1Database,
  account: {
    id: string;
    authHash: string;
    salt: string;
    kdfM: number;
    kdfT: number;
    kdfP: number;
    wrappedAccountKey: string;
    // Das Recovery-Wrapping haengt am Salt. Wechselt der Salt, muss es mitwandern,
    // sonst liefert /auth/recover einen neuen Salt zu einem alten Wrapping.
    recoveryAuthHash: string;
    wrappedAccountKeyRecovery: string;
  },
): Promise<void> {
  await db
    .prepare(
      `UPDATE accounts SET auth_hash = ?, salt = ?, kdf_m = ?, kdf_t = ?, kdf_p = ?, wrapped_account_key = ?,
        recovery_auth_hash = ?, wrapped_account_key_recovery = ?
       WHERE id = ?`,
    )
    .bind(
      account.authHash,
      account.salt,
      account.kdfM,
      account.kdfT,
      account.kdfP,
      account.wrappedAccountKey,
      account.recoveryAuthHash,
      account.wrappedAccountKeyRecovery,
      account.id,
    )
    .run();
}

export async function getCurrentSeq(db: D1Database, accountId: string): Promise<number> {
  const row = await db.prepare("SELECT value FROM seq WHERE account_id = ?").bind(accountId).first<{ value: number }>();
  return row?.value ?? 0;
}

export async function getStoredHlcs(
  db: D1Database,
  accountId: string,
  ids: string[],
): Promise<Map<string, string>> {
  const map = new Map<string, string>();
  if (ids.length === 0) return map;
  const placeholders = ids.map(() => "?").join(",");
  const { results } = await db
    .prepare(`SELECT id, hlc FROM records WHERE account_id = ? AND id IN (${placeholders})`)
    .bind(accountId, ...ids)
    .all<{ id: string; hlc: string }>();
  for (const row of results) map.set(row.id, row.hlc);
  return map;
}

export function upsertRecordStmt(
  db: D1Database,
  accountId: string,
  record: {
    id: string;
    kind: string;
    hlc: string;
    deviceId: string;
    deleted: boolean;
    formatVersion: number;
    nonce: string;
    ciphertext: string;
    serverSeq: number;
  },
): D1PreparedStatement {
  return db
    .prepare(
      `INSERT INTO records (account_id, id, kind, hlc, device_id, deleted, format_version, nonce, ciphertext, server_seq)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(account_id, id) DO UPDATE SET
         kind = excluded.kind,
         hlc = excluded.hlc,
         device_id = excluded.device_id,
         deleted = excluded.deleted,
         format_version = excluded.format_version,
         nonce = excluded.nonce,
         ciphertext = excluded.ciphertext,
         server_seq = excluded.server_seq
       WHERE excluded.hlc > records.hlc`,
    )
    .bind(
      accountId,
      record.id,
      record.kind,
      record.hlc,
      record.deviceId,
      record.deleted ? 1 : 0,
      record.formatVersion,
      record.nonce,
      record.ciphertext,
      record.serverSeq,
    );
}

/** Only ever raises the stored seq -- safe even if called with a stale value. */
export function bumpSeqStmt(db: D1Database, accountId: string, newValue: number): D1PreparedStatement {
  return db
    .prepare(
      `INSERT INTO seq (account_id, value) VALUES (?, ?)
       ON CONFLICT(account_id) DO UPDATE SET value = excluded.value WHERE excluded.value > seq.value`,
    )
    .bind(accountId, newValue);
}

export async function listRecordsSince(
  db: D1Database,
  accountId: string,
  since: number,
  limit: number,
): Promise<{ rows: RecordRow[]; hasMore: boolean }> {
  const { results } = await db
    .prepare(
      `SELECT * FROM records WHERE account_id = ? AND server_seq > ? ORDER BY server_seq ASC LIMIT ?`,
    )
    .bind(accountId, since, limit + 1)
    .all<RecordRow>();
  const hasMore = results.length > limit;
  return { rows: hasMore ? results.slice(0, limit) : results, hasMore };
}

export async function countRecords(db: D1Database, accountId: string): Promise<number> {
  const row = await db
    .prepare("SELECT COUNT(*) AS n FROM records WHERE account_id = ?")
    .bind(accountId)
    .first<{ n: number }>();
  return row?.n ?? 0;
}

export async function sumBlobBytes(db: D1Database, accountId: string): Promise<number> {
  const row = await db
    .prepare("SELECT COALESCE(SUM(size), 0) AS total FROM blobs WHERE account_id = ?")
    .bind(accountId)
    .first<{ total: number }>();
  return row?.total ?? 0;
}

export async function upsertBlobMeta(
  db: D1Database,
  accountId: string,
  id: string,
  size: number,
  now: number,
): Promise<void> {
  await db
    .prepare(
      `INSERT INTO blobs (account_id, id, size, created_at) VALUES (?, ?, ?, ?)
       ON CONFLICT(account_id, id) DO UPDATE SET size = excluded.size`,
    )
    .bind(accountId, id, size, now)
    .run();
}

export async function deleteBlobMeta(db: D1Database, accountId: string, id: string): Promise<void> {
  await db.prepare("DELETE FROM blobs WHERE account_id = ? AND id = ?").bind(accountId, id).run();
}
