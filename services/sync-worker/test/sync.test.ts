import { describe, expect, it } from "vitest";
import { api, hlc, makeEnvelope, randomB64, registerAccount } from "./helpers";

describe("sync push/pull", () => {
  it("round-trips a push through a pull", async () => {
    const { session_token, deviceId } = await registerAccount();
    const envelope = makeEnvelope({ device_id: deviceId, hlc: hlc(Date.now(), 1, deviceId) });

    const pushRes = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [envelope] }),
    });
    expect(pushRes.status).toBe(200);
    const pushBody = (await pushRes.json()) as { accepted: string[]; rejected: unknown[]; server_seq: number };
    expect(pushBody.accepted).toEqual([envelope.id]);
    expect(pushBody.rejected).toEqual([]);
    expect(pushBody.server_seq).toBe(1);

    const pullRes = await api("/v1/sync/pull?since=0", { token: session_token });
    expect(pullRes.status).toBe(200);
    const pullBody = (await pullRes.json()) as { records: Array<Record<string, unknown>>; next_seq: number; has_more: boolean };
    expect(pullBody.records).toHaveLength(1);
    expect(pullBody.records[0]).toMatchObject({
      id: envelope.id,
      kind: "note",
      ciphertext: envelope.ciphertext,
      server_seq: 1,
    });
    expect(pullBody.next_seq).toBe(1);
    expect(pullBody.has_more).toBe(false);
  });

  it("push is idempotent: an older or equal hlc is accepted but not written", async () => {
    const { session_token, deviceId } = await registerAccount();
    const id = crypto.randomUUID();
    const winningHlc = hlc(2_000_000_000_000, 5, deviceId);
    const olderHlc = hlc(1_000_000_000_000, 1, deviceId);

    const first = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [makeEnvelope({ id, device_id: deviceId, hlc: winningHlc, ciphertext: randomB64(8) })] }),
    });
    expect(first.status).toBe(200);

    const olderCiphertext = randomB64(8);
    const second = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({
        records: [makeEnvelope({ id, device_id: deviceId, hlc: olderHlc, ciphertext: olderCiphertext })],
      }),
    });
    expect(second.status).toBe(200);
    const secondBody = (await second.json()) as { accepted: string[]; rejected: unknown[] };
    // Accepted (structurally valid) but silently not written, per SYNC_PROTOCOL.md section 4.
    expect(secondBody.accepted).toEqual([id]);
    expect(secondBody.rejected).toEqual([]);

    const pull = await api("/v1/sync/pull?since=0", { token: session_token });
    const pullBody = (await pull.json()) as { records: Array<Record<string, unknown>> };
    const stored = pullBody.records.find((r) => r.id === id);
    expect(stored?.hlc).toBe(winningHlc);
    expect(stored?.ciphertext).not.toBe(olderCiphertext);
  });

  it("a later hlc overwrites the stored record", async () => {
    const { session_token, deviceId } = await registerAccount();
    const id = crypto.randomUUID();
    const first = hlc(1_000_000_000_000, 1, deviceId);
    const later = hlc(3_000_000_000_000, 1, deviceId);

    await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [makeEnvelope({ id, device_id: deviceId, hlc: first })] }),
    });
    const newCiphertext = randomB64(16);
    await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [makeEnvelope({ id, device_id: deviceId, hlc: later, ciphertext: newCiphertext })] }),
    });

    const pull = await api("/v1/sync/pull?since=0", { token: session_token });
    const pullBody = (await pull.json()) as { records: Array<Record<string, unknown>> };
    const stored = pullBody.records.find((r) => r.id === id);
    expect(stored?.hlc).toBe(later);
    expect(stored?.ciphertext).toBe(newCiphertext);
  });

  it("handles tombstones: deleted=true requires empty ciphertext and round-trips as deleted", async () => {
    const { session_token, deviceId } = await registerAccount();
    const id = crypto.randomUUID();
    const tombstone = makeEnvelope({ id, device_id: deviceId, deleted: true, ciphertext: "", nonce: "" });

    const pushRes = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [tombstone] }),
    });
    const pushBody = (await pushRes.json()) as { accepted: string[]; rejected: unknown[] };
    expect(pushBody.accepted).toEqual([id]);
    expect(pushBody.rejected).toEqual([]);

    const pull = await api("/v1/sync/pull?since=0", { token: session_token });
    const pullBody = (await pull.json()) as { records: Array<Record<string, unknown>> };
    const stored = pullBody.records.find((r) => r.id === id);
    expect(stored?.deleted).toBe(true);
    expect(stored?.ciphertext).toBe("");
  });

  it("rejects a tombstone whose ciphertext is not empty", async () => {
    const { session_token, deviceId } = await registerAccount();
    const bad = makeEnvelope({ device_id: deviceId, deleted: true, ciphertext: randomB64(4) });

    const res = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [bad] }),
    });
    const body = (await res.json()) as { accepted: string[]; rejected: Array<{ id: string; reason: string }> };
    expect(body.accepted).toEqual([]);
    expect(body.rejected).toEqual([{ id: bad.id, reason: "tombstone_ciphertext_not_empty" }]);
  });

  it("rejects records with an invalid kind or malformed hlc, without failing the whole push", async () => {
    const { session_token, deviceId } = await registerAccount();
    const good = makeEnvelope({ device_id: deviceId });
    const badKind = makeEnvelope({ device_id: deviceId, kind: "not-a-kind" });
    const badHlc = makeEnvelope({ device_id: deviceId, hlc: "not-well-formed" });

    const res = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [good, badKind, badHlc] }),
    });
    expect(res.status).toBe(200);
    const body = (await res.json()) as { accepted: string[]; rejected: Array<{ id: string; reason: string }> };
    expect(body.accepted).toEqual([good.id]);
    expect(body.rejected).toEqual(
      expect.arrayContaining([
        { id: badKind.id, reason: "invalid_kind" },
        { id: badHlc.id, reason: "invalid_hlc" },
      ]),
    );
  });

  it("413s a push with more than 500 records", async () => {
    const { session_token, deviceId } = await registerAccount();
    const records = Array.from({ length: 501 }, () => makeEnvelope({ device_id: deviceId }));
    const res = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records }),
    });
    expect(res.status).toBe(413);
  });

  it("paginates pull with since/limit and has_more", async () => {
    const { session_token, deviceId } = await registerAccount();
    const total = 5;
    const records = Array.from({ length: total }, (_, i) =>
      makeEnvelope({ device_id: deviceId, hlc: hlc(1_000_000_000_000 + i, 0, deviceId) }),
    );
    const pushRes = await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records }),
    });
    expect((await pushRes.json<{ server_seq: number }>()).server_seq).toBe(total);

    const page1 = await api("/v1/sync/pull?since=0&limit=2", { token: session_token });
    const page1Body = (await page1.json()) as { records: unknown[]; next_seq: number; has_more: boolean };
    expect(page1Body.records).toHaveLength(2);
    expect(page1Body.has_more).toBe(true);
    expect(page1Body.next_seq).toBe(2);

    const page2 = await api(`/v1/sync/pull?since=${page1Body.next_seq}&limit=2`, { token: session_token });
    const page2Body = (await page2.json()) as { records: unknown[]; next_seq: number; has_more: boolean };
    expect(page2Body.records).toHaveLength(2);
    expect(page2Body.has_more).toBe(true);
    expect(page2Body.next_seq).toBe(4);

    const page3 = await api(`/v1/sync/pull?since=${page2Body.next_seq}&limit=2`, { token: session_token });
    const page3Body = (await page3.json()) as { records: unknown[]; next_seq: number; has_more: boolean };
    expect(page3Body.records).toHaveLength(1);
    expect(page3Body.has_more).toBe(false);
    expect(page3Body.next_seq).toBe(5);
  });

  it("status reports server_seq, record_count and blob_bytes", async () => {
    const { session_token, deviceId } = await registerAccount();
    await api("/v1/sync/push", {
      method: "POST",
      token: session_token,
      body: JSON.stringify({ records: [makeEnvelope({ device_id: deviceId }), makeEnvelope({ device_id: deviceId })] }),
    });

    const res = await api("/v1/sync/status", { token: session_token });
    expect(res.status).toBe(200);
    const body = (await res.json()) as { server_seq: number; record_count: number; blob_bytes: number };
    expect(body.server_seq).toBe(2);
    expect(body.record_count).toBe(2);
    expect(body.blob_bytes).toBe(0);
  });
});

describe("sync auth rejection", () => {
  it("rejects push/pull/status without a valid session", async () => {
    const push = await api("/v1/sync/push", { method: "POST", body: JSON.stringify({ records: [] }) });
    expect(push.status).toBe(401);
    const pull = await api("/v1/sync/pull");
    expect(pull.status).toBe(401);
    const status = await api("/v1/sync/status");
    expect(status.status).toBe(401);
  });
});

describe("blobs", () => {
  it("puts, gets, and deletes a blob", async () => {
    const { session_token } = await registerAccount();
    const id = crypto.randomUUID();
    const payload = new Uint8Array([1, 2, 3, 4, 5]);

    const put = await api(`/v1/blobs/${id}`, {
      method: "PUT",
      token: session_token,
      body: payload,
      headers: { "content-length": String(payload.byteLength), "content-type": "application/octet-stream" },
    });
    expect(put.status).toBe(201);

    const get = await api(`/v1/blobs/${id}`, { token: session_token });
    expect(get.status).toBe(200);
    const received = new Uint8Array(await get.arrayBuffer());
    expect(Array.from(received)).toEqual(Array.from(payload));

    const statusRes = await api("/v1/sync/status", { token: session_token });
    const statusBody = (await statusRes.json()) as { blob_bytes: number };
    expect(statusBody.blob_bytes).toBe(payload.byteLength);

    const del = await api(`/v1/blobs/${id}`, { method: "DELETE", token: session_token });
    expect(del.status).toBe(204);

    const afterDelete = await api(`/v1/blobs/${id}`, { token: session_token });
    expect(afterDelete.status).toBe(404);
  });

  it("404s a GET for a missing blob", async () => {
    const { session_token } = await registerAccount();
    const res = await api(`/v1/blobs/${crypto.randomUUID()}`, { token: session_token });
    expect(res.status).toBe(404);
  });
});
