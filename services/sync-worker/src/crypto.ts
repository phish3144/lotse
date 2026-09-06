/**
 * WebCrypto-only helpers. No runtime dependency: everything the Worker needs
 * (random bytes, SHA-256, PBKDF2, base64, ULIDs) comes from the platform.
 *
 * Server-side password hashing: THREAT_MODEL.md's key hierarchy uses
 * Argon2id client-side; SYNC_PROTOCOL.md section 8 explicitly allows the
 * *server* to fall back to PBKDF2-SHA256 (600 000 iterations) because a
 * Worker request has only 10 ms CPU and cannot run Argon2id. Security still
 * rests on the client-side Argon2id stretch -- this is defense in depth
 * against a raw D1 dump, not the primary defense.
 */

const PBKDF2_ITERATIONS = 600_000;
const SALT_BYTES = 16;
const ULID_ENCODING = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"; // Crockford base32, no I L O U

export function randomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return bytes;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize);
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

/** Decodes base64 to bytes. Throws (native DOMException) on invalid input. */
export function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/** Non-throwing base64 decode, for validating untrusted client input. */
export function tryBase64ToBytes(b64: string): Uint8Array | null {
  try {
    return base64ToBytes(b64);
  } catch {
    return null;
  }
}

export function bytesToBase64Url(bytes: Uint8Array): string {
  return bytesToBase64(bytes).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export function bytesToHex(bytes: Uint8Array): string {
  let hex = "";
  for (const b of bytes) {
    hex += b.toString(16).padStart(2, "0");
  }
  return hex;
}

export async function sha256Hex(input: string): Promise<string> {
  const data = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", data);
  return bytesToHex(new Uint8Array(digest));
}

/** Constant-time comparison for equal-length secrets (hashes, tokens). */
export function timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a[i]! ^ b[i]!;
  }
  return diff === 0;
}

/**
 * Hashes a base64-encoded secret (the client-derived `auth_key` or
 * `recovery_auth_key`) for storage. Format: `pbkdf2$<iterations>$<salt_b64>$<hash_b64>`.
 */
export async function hashSecret(secretB64: string): Promise<string> {
  const secretBytes = base64ToBytes(secretB64);
  const salt = randomBytes(SALT_BYTES);
  const hash = await deriveBits(secretBytes, salt, PBKDF2_ITERATIONS);
  return `pbkdf2$${PBKDF2_ITERATIONS}$${bytesToBase64(salt)}$${bytesToBase64(hash)}`;
}

/** Verifies a base64-encoded secret against a stored `hashSecret` string. Constant-time. */
export async function verifySecret(stored: string, secretB64: string): Promise<boolean> {
  const parts = stored.split("$");
  if (parts.length !== 4 || parts[0] !== "pbkdf2") return false;
  const iterations = Number(parts[1]);
  if (!Number.isInteger(iterations) || iterations <= 0) return false;
  const salt = tryBase64ToBytes(parts[2]!);
  const expected = tryBase64ToBytes(parts[3]!);
  const secretBytes = tryBase64ToBytes(secretB64);
  if (!salt || !expected || !secretBytes) return false;
  const actual = await deriveBits(secretBytes, salt, iterations);
  return timingSafeEqual(actual, expected);
}

async function deriveBits(secret: Uint8Array, salt: Uint8Array, iterations: number): Promise<Uint8Array> {
  const key = await crypto.subtle.importKey("raw", secret, "PBKDF2", false, ["deriveBits"]);
  const bits = await crypto.subtle.deriveBits(
    { name: "PBKDF2", hash: "SHA-256", salt, iterations },
    key,
    256,
  );
  return new Uint8Array(bits);
}

/** 32 random bytes, base64url-encoded -- the raw session token handed to clients. */
export function generateSessionToken(): string {
  return bytesToBase64Url(randomBytes(32));
}

/** Server-assigned account ID: a ULID (time-ordered, Crockford base32, 26 chars). */
export function generateUlid(): string {
  return encodeTime(Date.now(), 10) + encodeRandom(16);
}

function encodeTime(time: number, length: number): string {
  let t = time;
  let str = "";
  for (let i = 0; i < length; i++) {
    const mod = t % 32;
    str = ULID_ENCODING[mod] + str;
    t = (t - mod) / 32;
  }
  return str;
}

function encodeRandom(length: number): string {
  const bytes = randomBytes(length);
  let str = "";
  for (let i = 0; i < length; i++) {
    // 256 is divisible by 32, so `% 32` on a uniform byte is unbiased.
    str += ULID_ENCODING[bytes[i]! % 32];
  }
  return str;
}
