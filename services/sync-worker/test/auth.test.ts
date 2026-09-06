import { describe, expect, it } from "vitest";
import { api, registerAccount, registerBody } from "./helpers";

describe("auth", () => {
  it("registers, then logs in with the same auth_key", async () => {
    const { requestBody } = await registerAccount();

    const res = await api("/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({
        email: requestBody.email,
        auth_key: requestBody.auth_key,
        device: requestBody.device,
      }),
    });
    expect(res.status).toBe(200);
    const body = (await res.json()) as Record<string, unknown>;
    expect(body.account_id).toBeTypeOf("string");
    expect(body.session_token).toBeTypeOf("string");
    expect(body.wrapped_account_key).toBe(requestBody.wrapped_account_key);
    expect((body.kdf as Record<string, unknown>).m).toBe(65536);
  });

  it("rejects a duplicate email on register with 409", async () => {
    const { requestBody } = await registerAccount();
    const res = await api("/v1/auth/register", {
      method: "POST",
      body: JSON.stringify(registerBody({ email: requestBody.email })),
    });
    expect(res.status).toBe(409);
    const body = (await res.json()) as Record<string, unknown>;
    expect(body.error).toBe("email_taken");
  });

  it("rejects login with a wrong auth_key", async () => {
    const { requestBody } = await registerAccount();
    const res = await api("/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({
        email: requestBody.email,
        auth_key: "wrong-key-not-even-base64!!",
        device: requestBody.device,
      }),
    });
    expect(res.status).toBe(401);
  });

  it("prelogin returns salt and kdf for a known email, 404 for unknown", async () => {
    const { requestBody } = await registerAccount();
    const ok = await api(`/v1/auth/prelogin?email=${encodeURIComponent(requestBody.email as string)}`);
    expect(ok.status).toBe(200);
    const body = (await ok.json()) as Record<string, unknown>;
    expect(body.salt).toBe(requestBody.salt);

    const missing = await api(`/v1/auth/prelogin?email=nobody-${crypto.randomUUID()}@example.com`);
    expect(missing.status).toBe(404);
  });

  it("rejects protected endpoints without a bearer token", async () => {
    const res = await api("/v1/devices");
    expect(res.status).toBe(401);
  });

  it("rejects protected endpoints with a garbage bearer token", async () => {
    const res = await api("/v1/devices", { token: "not-a-real-token" });
    expect(res.status).toBe(401);
  });

  it("logout revokes the session token", async () => {
    const { session_token } = await registerAccount();
    const before = await api("/v1/devices", { token: session_token });
    expect(before.status).toBe(200);

    const logoutRes = await api("/v1/auth/logout", { method: "POST", token: session_token });
    expect(logoutRes.status).toBe(204);

    const after = await api("/v1/devices", { token: session_token });
    expect(after.status).toBe(401);
  });

  it("lists the registering device and deleting it revokes its session", async () => {
    const { session_token, deviceId } = await registerAccount();

    const list = await api("/v1/devices", { token: session_token });
    expect(list.status).toBe(200);
    const devices = (await list.json()) as Array<{ id: string }>;
    expect(devices.map((d) => d.id)).toContain(deviceId);

    const del = await api(`/v1/devices/${deviceId}`, { method: "DELETE", token: session_token });
    expect(del.status).toBe(204);

    // The session belonged to the now-deleted device.
    const after = await api("/v1/devices", { token: session_token });
    expect(after.status).toBe(401);
  });

  it("changes the password and revokes other sessions but keeps the current one", async () => {
    const { requestBody } = await registerAccount();

    // A second login session for the same account/device.
    const loginRes = await api("/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({
        email: requestBody.email,
        auth_key: requestBody.auth_key,
        device: { ...(requestBody.device as object), id: crypto.randomUUID() },
      }),
    });
    const { session_token: otherSession } = (await loginRes.json()) as { session_token: string };

    const thirdLogin = await api("/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({
        email: requestBody.email,
        auth_key: requestBody.auth_key,
        device: { ...(requestBody.device as object), id: crypto.randomUUID() },
      }),
    });
    const { session_token: currentSession } = (await thirdLogin.json()) as { session_token: string };

    const changeRes = await api("/v1/auth/password", {
      method: "POST",
      token: currentSession,
      body: JSON.stringify({
        old_auth_key: requestBody.auth_key,
        new_auth_key: "bmV3LWF1dGgta2V5LWZvci10ZXN0aW5nLTMyYg==",
        new_salt: "bmV3LXNhbHQ=",
        new_kdf: { m: 65536, t: 4, p: 1 },
        wrapped_account_key: requestBody.wrapped_account_key,
      }),
    });
    expect(changeRes.status).toBe(204);

    const otherAfter = await api("/v1/devices", { token: otherSession });
    expect(otherAfter.status).toBe(401);

    const currentAfter = await api("/v1/devices", { token: currentSession });
    expect(currentAfter.status).toBe(200);
  });

  it("recover returns the wrapped recovery key for a valid recovery_auth_key", async () => {
    const { requestBody } = await registerAccount();
    const res = await api("/v1/auth/recover", {
      method: "POST",
      body: JSON.stringify({ email: requestBody.email, recovery_auth_key: requestBody.recovery_auth_key }),
    });
    expect(res.status).toBe(200);
    const body = (await res.json()) as Record<string, unknown>;
    expect(body.wrapped_account_key_recovery).toBe(requestBody.wrapped_account_key_recovery);

    const bad = await api("/v1/auth/recover", {
      method: "POST",
      body: JSON.stringify({ email: requestBody.email, recovery_auth_key: "d2hhdGV2ZXI=" }),
    });
    expect(bad.status).toBe(401);
  });
});
