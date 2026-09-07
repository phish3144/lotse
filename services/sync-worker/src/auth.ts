/**
 * Account and session handlers: SYNC_PROTOCOL.md section 4 "Konto" table.
 *
 * Rate limiting for these endpoints is deliberately NOT implemented here.
 * The task and SYNC_PROTOCOL.md section 8 call for per-IP limiting on
 * auth endpoints to live in front of the Worker as a Cloudflare WAF rate
 * limiting rule (e.g. on `/v1/auth/*`), not in Worker code, since it should
 * stop abusive requests before they burn CPU on PBKDF2.
 */
import {
  deleteAllSessions,
  deleteDevice,
  deleteOtherSessions,
  deleteSessionByTokenHash,
  deleteSessionsForDevice,
  getAccountByEmail,
  getAccountById,
  getDevice,
  getSessionByTokenHash,
  insertAccountStmt,
  insertSeqRowStmt,
  insertSessionStmt,
  listDevices,
  sessionDurationMs,
  touchDeviceLastSeen,
  updateAccountPassword,
  upsertDeviceStmt,
} from "./db";
import { badRequest, conflict, json, noContent, unauthorized, ApiError } from "./errors";
import { generateSessionToken, generateUlid, hashSecret, sha256Hex, verifySecret } from "./crypto";
import { isPlatform, type AuthContext, type DeviceInput, type Env, type KdfParams, type Platform, type RequestContext } from "./types";

// ---------------------------------------------------------------------------
// Shared body-parsing helpers
// ---------------------------------------------------------------------------

async function parseJson(request: Request): Promise<Record<string, unknown>> {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    throw badRequest("invalid JSON body", "invalid_json");
  }
  if (typeof body !== "object" || body === null || Array.isArray(body)) {
    throw badRequest("expected a JSON object body", "invalid_json");
  }
  return body as Record<string, unknown>;
}

function requireString(body: Record<string, unknown>, field: string): string {
  const value = body[field];
  if (typeof value !== "string" || value.length === 0) {
    throw badRequest(`missing or invalid field: ${field}`, "invalid_field");
  }
  return value;
}

function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

function requireKdf(body: Record<string, unknown>, field: string): KdfParams {
  const value = body[field];
  if (typeof value !== "object" || value === null) {
    throw badRequest(`missing or invalid field: ${field}`, "invalid_field");
  }
  const { m, t, p } = value as Record<string, unknown>;
  if (
    typeof m !== "number" || !Number.isInteger(m) || m <= 0 ||
    typeof t !== "number" || !Number.isInteger(t) || t <= 0 ||
    typeof p !== "number" || !Number.isInteger(p) || p <= 0
  ) {
    throw badRequest(`invalid kdf parameters in field: ${field}`, "invalid_field");
  }
  return { m, t, p };
}

function requireDevice(body: Record<string, unknown>): DeviceInput {
  const value = body.device;
  if (typeof value !== "object" || value === null) {
    throw badRequest("missing or invalid field: device", "invalid_field");
  }
  const { id, name, platform } = value as Record<string, unknown>;
  if (typeof id !== "string" || id.length === 0) {
    throw badRequest("missing or invalid field: device.id", "invalid_field");
  }
  if (typeof name !== "string" || name.length === 0) {
    throw badRequest("missing or invalid field: device.name", "invalid_field");
  }
  if (!isPlatform(platform)) {
    throw badRequest("device.platform must be one of desktop|web|cli", "invalid_field");
  }
  return { id, name, platform: platform as Platform };
}

async function requireBearerToken(request: Request): Promise<string> {
  const header = request.headers.get("authorization") ?? request.headers.get("Authorization");
  if (!header || !header.startsWith("Bearer ")) {
    throw unauthorized("missing bearer token");
  }
  const token = header.slice("Bearer ".length).trim();
  if (token.length === 0) {
    throw unauthorized("missing bearer token");
  }
  return token;
}

// ---------------------------------------------------------------------------
// Auth middleware (used by the router for every protected route)
// ---------------------------------------------------------------------------

export async function requireAuth(request: Request, env: Env): Promise<AuthContext> {
  const token = await requireBearerToken(request);
  const tokenHash = await sha256Hex(token);
  const session = await getSessionByTokenHash(env.DB, tokenHash);
  if (!session) {
    throw unauthorized("invalid session token", "invalid_token");
  }
  const now = Date.now();
  if (session.expires_at <= now) {
    // Best-effort cleanup; failure here must not block the 401 response.
    await deleteSessionByTokenHash(env.DB, tokenHash).catch(() => undefined);
    throw unauthorized("session expired", "expired_token");
  }
  // Throttled inside touchDeviceLastSeen: a no-op UPDATE (0 rows) when the
  // device was already touched within LAST_SEEN_THROTTLE_MS.
  await touchDeviceLastSeen(env.DB, session.account_id, session.device_id, now);
  return { accountId: session.account_id, deviceId: session.device_id, tokenHash };
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

export async function prelogin({ request, env }: RequestContext): Promise<Response> {
  const url = new URL(request.url);
  const emailParam = url.searchParams.get("email");
  if (!emailParam) {
    throw badRequest("missing query parameter: email", "invalid_field");
  }
  const account = await getAccountByEmail(env.DB, normalizeEmail(emailParam));
  if (!account) {
    throw new ApiError(404, "not_found", "no account for this email");
  }
  return json({ salt: account.salt, kdf: { m: account.kdf_m, t: account.kdf_t, p: account.kdf_p } });
}

export async function register({ request, env }: RequestContext): Promise<Response> {
  const body = await parseJson(request);
  const email = normalizeEmail(requireString(body, "email"));
  const authKey = requireString(body, "auth_key");
  // Not in the section-4 request table, but required to satisfy the
  // NOT NULL `recovery_auth_hash` column and the prose in THREAT_MODEL.md
  // ("der Dienst speichert dessen Argon2id-Hash bei der Registrierung mit").
  const recoveryAuthKey = requireString(body, "recovery_auth_key");
  const salt = requireString(body, "salt");
  const kdf = requireKdf(body, "kdf");
  const wrappedAccountKey = requireString(body, "wrapped_account_key");
  const wrappedAccountKeyRecovery = requireString(body, "wrapped_account_key_recovery");
  const device = requireDevice(body);

  const existing = await getAccountByEmail(env.DB, email);
  if (existing) {
    throw conflict("an account with this email already exists", "email_taken");
  }

  const accountId = generateUlid();
  const [authHash, recoveryHash] = await Promise.all([hashSecret(authKey), hashSecret(recoveryAuthKey)]);
  const now = Date.now();
  const sessionToken = generateSessionToken();
  const tokenHash = await sha256Hex(sessionToken);
  const expiresAt = now + sessionDurationMs(device.platform);

  await env.DB.batch([
    insertAccountStmt(env.DB, {
      id: accountId,
      email,
      authHash,
      recoveryAuthHash: recoveryHash,
      salt,
      kdfM: kdf.m,
      kdfT: kdf.t,
      kdfP: kdf.p,
      wrappedAccountKey,
      wrappedAccountKeyRecovery,
      createdAt: now,
    }),
    upsertDeviceStmt(env.DB, { id: device.id, accountId, name: device.name, platform: device.platform, now }),
    insertSessionStmt(env.DB, { tokenHash, accountId, deviceId: device.id, now, expiresAt }),
    insertSeqRowStmt(env.DB, accountId),
  ]);

  return json({ account_id: accountId, session_token: sessionToken }, 201);
}

export async function login({ request, env }: RequestContext): Promise<Response> {
  const body = await parseJson(request);
  const email = normalizeEmail(requireString(body, "email"));
  const authKey = requireString(body, "auth_key");
  const device = requireDevice(body);

  const account = await getAccountByEmail(env.DB, email);
  if (!account) {
    throw unauthorized("invalid email or auth_key", "invalid_credentials");
  }
  const valid = await verifySecret(account.auth_hash, authKey);
  if (!valid) {
    throw unauthorized("invalid email or auth_key", "invalid_credentials");
  }

  const now = Date.now();
  const sessionToken = generateSessionToken();
  const tokenHash = await sha256Hex(sessionToken);
  const expiresAt = now + sessionDurationMs(device.platform);

  await env.DB.batch([
    upsertDeviceStmt(env.DB, {
      id: device.id,
      accountId: account.id,
      name: device.name,
      platform: device.platform,
      now,
    }),
    insertSessionStmt(env.DB, { tokenHash, accountId: account.id, deviceId: device.id, now, expiresAt }),
  ]);

  return json({
    account_id: account.id,
    session_token: sessionToken,
    salt: account.salt,
    kdf: { m: account.kdf_m, t: account.kdf_t, p: account.kdf_p },
    wrapped_account_key: account.wrapped_account_key,
  });
}

export async function logout({ request, env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  await deleteSessionByTokenHash(env.DB, auth.tokenHash);
  void request;
  return noContent();
}

export async function changePassword({ request, env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const body = await parseJson(request);
  const oldAuthKey = requireString(body, "old_auth_key");
  const newAuthKey = requireString(body, "new_auth_key");
  const newSalt = requireString(body, "new_salt");
  const newKdf = requireKdf(body, "new_kdf");
  const wrappedAccountKey = requireString(body, "wrapped_account_key");
  // Pflicht: der Client rechnet das Recovery-Wrapping mit dem neuen Salt neu. Ohne das
  // passt der gespeicherte Wiederherstellungscode nicht mehr zum Konto.
  const newRecoveryAuthKey = requireString(body, "recovery_auth_key");
  const wrappedAccountKeyRecovery = requireString(body, "wrapped_account_key_recovery");

  const account = await getAccountById(env.DB, auth.accountId);
  if (!account) throw unauthorized("invalid session");
  const valid = await verifySecret(account.auth_hash, oldAuthKey);
  if (!valid) {
    throw unauthorized("old_auth_key does not match", "invalid_credentials");
  }
  const newHash = await hashSecret(newAuthKey);
  const newRecoveryHash = await hashSecret(newRecoveryAuthKey);

  await updateAccountPassword(env.DB, {
    id: account.id,
    authHash: newHash,
    salt: newSalt,
    kdfM: newKdf.m,
    kdfT: newKdf.t,
    kdfP: newKdf.p,
    wrappedAccountKey,
    recoveryAuthHash: newRecoveryHash,
    wrappedAccountKeyRecovery,
  });
  // "widerruft alle anderen Sitzungen" -- every session except the one used
  // to make this very request.
  await deleteOtherSessions(env.DB, account.id, auth.tokenHash);
  return noContent();
}

export async function recover({ request, env }: RequestContext): Promise<Response> {
  const body = await parseJson(request);
  const email = normalizeEmail(requireString(body, "email"));
  const recoveryAuthKey = requireString(body, "recovery_auth_key");

  const account = await getAccountByEmail(env.DB, email);
  if (!account) {
    throw unauthorized("invalid email or recovery_auth_key", "invalid_recovery");
  }
  const valid = await verifySecret(account.recovery_auth_hash, recoveryAuthKey);
  if (!valid) {
    throw unauthorized("invalid email or recovery_auth_key", "invalid_recovery");
  }

  return json({
    wrapped_account_key_recovery: account.wrapped_account_key_recovery,
    salt: account.salt,
    kdf: { m: account.kdf_m, t: account.kdf_t, p: account.kdf_p },
  });
}

/**
 * Schliesst eine Wiederherstellung ab: neue Zugangsdaten, beglaubigt mit dem
 * recovery_auth_key statt mit dem alten Passwort.
 *
 * Ohne diesen Weg gaebe es keinen: /auth/password verlangt den old_auth_key, und den
 * hat nicht, wer sein Passwort vergessen hat. Das Geraet haette danach ein neues
 * Passwort, der Dienst weiter das alte -- die Aussperrung faellt erst beim naechsten
 * Login auf einem zweiten Geraet auf.
 */
export async function recoverComplete({ request, env }: RequestContext): Promise<Response> {
  const body = await parseJson(request);
  const email = normalizeEmail(requireString(body, "email"));
  const recoveryAuthKey = requireString(body, "recovery_auth_key");
  const newAuthKey = requireString(body, "new_auth_key");
  const newSalt = requireString(body, "new_salt");
  const newKdf = requireKdf(body, "new_kdf");
  const wrappedAccountKey = requireString(body, "wrapped_account_key");
  const newRecoveryAuthKey = requireString(body, "new_recovery_auth_key");
  const wrappedAccountKeyRecovery = requireString(body, "wrapped_account_key_recovery");

  const account = await getAccountByEmail(env.DB, email);
  if (!account) {
    throw unauthorized("invalid email or recovery_auth_key", "invalid_recovery");
  }
  const valid = await verifySecret(account.recovery_auth_hash, recoveryAuthKey);
  if (!valid) {
    throw unauthorized("invalid email or recovery_auth_key", "invalid_recovery");
  }

  await updateAccountPassword(env.DB, {
    id: account.id,
    authHash: await hashSecret(newAuthKey),
    salt: newSalt,
    kdfM: newKdf.m,
    kdfT: newKdf.t,
    kdfP: newKdf.p,
    wrappedAccountKey,
    recoveryAuthHash: await hashSecret(newRecoveryAuthKey),
    wrappedAccountKeyRecovery,
  });
  // Wer das Passwort vergessen hat, weiss nicht, wer sonst noch angemeldet ist.
  await deleteAllSessions(env.DB, account.id);
  return noContent();
}

export async function listDevicesHandler({ env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const devices = await listDevices(env.DB, auth.accountId);
  return json(
    devices.map((d) => ({
      id: d.id,
      name: d.name,
      platform: d.platform,
      created_at: d.created_at,
      last_seen_at: d.last_seen_at,
    })),
  );
}

export async function deleteDeviceHandler({ env, auth, params }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const deviceId = params.id;
  if (!deviceId) throw badRequest("missing device id", "invalid_field");
  const device = await getDevice(env.DB, auth.accountId, deviceId);
  if (!device) {
    throw new ApiError(404, "not_found", "no such device");
  }
  await deleteSessionsForDevice(env.DB, auth.accountId, deviceId);
  await deleteDevice(env.DB, auth.accountId, deviceId);
  return noContent();
}
