/**
 * /v1/sync/push, /v1/sync/pull, /v1/sync/status -- SYNC_PROTOCOL.md section 4
 * "Sync" and section 2 "Datensatz-Umschlag".
 */
import {
  bumpSeqStmt,
  countRecords,
  getCurrentSeq,
  getStoredHlcs,
  listRecordsSince,
  sumBlobBytes,
  upsertRecordStmt,
} from "./db";
import { badRequest, json, payloadTooLarge, unauthorized } from "./errors";
import { tryBase64ToBytes } from "./crypto";
import { HLC_PATTERN, isRecordKind, type InboundEnvelope, type OutboundEnvelope, type RequestContext } from "./types";

const MAX_RECORDS_PER_PUSH = 500;
const MAX_CIPHERTEXT_BYTES = 256 * 1024;
const NONCE_BYTES = 24;

const DEFAULT_PULL_LIMIT = 500;
const MAX_PULL_LIMIT = 1000;

interface ValidationFailure {
  id: string;
  reason: string;
}

interface ValidationResult {
  valid: InboundEnvelope[];
  rejected: ValidationFailure[];
}

/**
 * Structural validation only, per SYNC_PROTOCOL.md section 2: "Der Dienst
 * prüft nur: Base64 gültig, ciphertext <= 256 KiB, kind aus der erlaubten
 * Liste, hlc wohlgeformt" -- plus the tombstone-ciphertext-empty rule this
 * task adds explicitly. Record IDs and device IDs are otherwise trusted
 * as opaque strings; the service never interprets plaintext.
 */
function validateEnvelope(raw: unknown): { ok: true; value: InboundEnvelope } | { ok: false; id: string; reason: string } {
  if (typeof raw !== "object" || raw === null) {
    return { ok: false, id: "?", reason: "not_an_object" };
  }
  const r = raw as Record<string, unknown>;
  const id = typeof r.id === "string" && r.id.length > 0 ? r.id : undefined;
  const fail = (reason: string) => ({ ok: false as const, id: id ?? "?", reason });

  if (!id) return fail("missing_id");
  if (typeof r.kind !== "string" || !isRecordKind(r.kind)) return fail("invalid_kind");
  if (typeof r.hlc !== "string" || !HLC_PATTERN.test(r.hlc)) return fail("invalid_hlc");
  if (typeof r.device_id !== "string" || r.device_id.length === 0) return fail("missing_device_id");
  if (typeof r.deleted !== "boolean") return fail("invalid_deleted");
  if (typeof r.format_version !== "number" || !Number.isInteger(r.format_version) || r.format_version < 1) {
    return fail("invalid_format_version");
  }
  if (typeof r.nonce !== "string") return fail("invalid_nonce");
  const nonceBytes = tryBase64ToBytes(r.nonce);
  if (!nonceBytes || nonceBytes.length !== NONCE_BYTES) return fail("invalid_nonce");
  if (typeof r.ciphertext !== "string") return fail("invalid_ciphertext");
  const ciphertextBytes = tryBase64ToBytes(r.ciphertext);
  if (!ciphertextBytes) return fail("invalid_ciphertext");
  if (ciphertextBytes.length > MAX_CIPHERTEXT_BYTES) return fail("ciphertext_too_large");
  if (r.deleted && ciphertextBytes.length !== 0) return fail("tombstone_ciphertext_not_empty");

  return {
    ok: true,
    value: {
      id,
      kind: r.kind,
      hlc: r.hlc,
      device_id: r.device_id,
      deleted: r.deleted,
      format_version: r.format_version,
      nonce: r.nonce,
      ciphertext: r.ciphertext,
    },
  };
}

function validateEnvelopes(records: unknown[]): ValidationResult {
  const valid: InboundEnvelope[] = [];
  const rejected: ValidationFailure[] = [];
  for (const raw of records) {
    const result = validateEnvelope(raw);
    if (result.ok) valid.push(result.value);
    else rejected.push({ id: result.id, reason: result.reason });
  }
  return { valid, rejected };
}

export async function push({ request, env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    throw badRequest("invalid JSON body", "invalid_json");
  }
  if (typeof body !== "object" || body === null || !Array.isArray((body as Record<string, unknown>).records)) {
    throw badRequest("expected { records: [...] }", "invalid_json");
  }
  const records = (body as { records: unknown[] }).records;
  if (records.length > MAX_RECORDS_PER_PUSH) {
    throw payloadTooLarge(`at most ${MAX_RECORDS_PER_PUSH} records per push`, "too_many_records");
  }

  const { valid, rejected } = validateEnvelopes(records);
  const acceptedIds: string[] = valid.map((v) => v.id);

  const currentSeq = await getCurrentSeq(env.DB, auth.accountId);

  // Idempotency (SYNC_PROTOCOL.md section 4): an envelope whose hlc is <=
  // the stored one is reported accepted but not written.
  const storedHlcs = await getStoredHlcs(env.DB, auth.accountId, valid.map((v) => v.id));
  const toWrite = valid.filter((v) => {
    const stored = storedHlcs.get(v.id);
    return stored === undefined || v.hlc > stored;
  });

  let finalSeq = currentSeq;
  if (toWrite.length > 0) {
    const statements = toWrite.map((record, index) => {
      const serverSeq = currentSeq + index + 1;
      return upsertRecordStmt(env.DB, auth.accountId, {
        id: record.id,
        kind: record.kind,
        hlc: record.hlc,
        deviceId: record.device_id,
        deleted: record.deleted,
        formatVersion: record.format_version,
        nonce: record.nonce,
        ciphertext: record.ciphertext,
        serverSeq,
      });
    });
    finalSeq = currentSeq + toWrite.length;
    statements.push(bumpSeqStmt(env.DB, auth.accountId, finalSeq));
    // Single D1 batch: all record upserts + the seq bump commit atomically.
    await env.DB.batch(statements);
  }

  return json({ accepted: acceptedIds, rejected, server_seq: finalSeq });
}

export async function pull({ request, env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const url = new URL(request.url);

  const sinceParam = url.searchParams.get("since");
  let since = 0;
  if (sinceParam !== null) {
    since = Number(sinceParam);
    if (!Number.isInteger(since) || since < 0) {
      throw badRequest("invalid query parameter: since", "invalid_field");
    }
  }

  const limitParam = url.searchParams.get("limit");
  let limit = DEFAULT_PULL_LIMIT;
  if (limitParam !== null) {
    limit = Number(limitParam);
    if (!Number.isInteger(limit) || limit < 1) {
      throw badRequest("invalid query parameter: limit", "invalid_field");
    }
    limit = Math.min(limit, MAX_PULL_LIMIT);
  }

  const { rows, hasMore } = await listRecordsSince(env.DB, auth.accountId, since, limit);
  const outRecords: OutboundEnvelope[] = rows.map((row) => ({
    id: row.id,
    kind: row.kind as InboundEnvelope["kind"],
    hlc: row.hlc,
    device_id: row.device_id,
    deleted: row.deleted !== 0,
    format_version: row.format_version,
    nonce: row.nonce,
    ciphertext: row.ciphertext,
    server_seq: row.server_seq,
  }));
  const lastRow = rows[rows.length - 1];
  const nextSeq = lastRow ? lastRow.server_seq : since;

  return json({ records: outRecords, next_seq: nextSeq, has_more: hasMore });
}

export async function status({ env, auth }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const [serverSeq, recordCount, blobBytes] = await Promise.all([
    getCurrentSeq(env.DB, auth.accountId),
    countRecords(env.DB, auth.accountId),
    sumBlobBytes(env.DB, auth.accountId),
  ]);
  return json({ server_seq: serverSeq, record_count: recordCount, blob_bytes: blobBytes });
}
