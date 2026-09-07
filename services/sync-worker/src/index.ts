/**
 * Lotse sync-worker entry point. Routes every endpoint of
 * docs/SYNC_PROTOCOL.md section 4. The service stores only ciphertext and
 * metadata (docs/THREAT_MODEL.md section 3) -- never log request bodies or
 * tokens (see src/errors.ts#errorResponse for the one place errors are
 * logged, deliberately without echoing input).
 */
import {
  changePassword,
  deleteDeviceHandler,
  listDevicesHandler,
  login,
  logout,
  prelogin,
  recover,
  recoverComplete,
  register,
} from "./auth";
import { deleteBlob, getBlob, putBlob } from "./blobs";
import { Router } from "./router";
import { pull, push, status } from "./sync";
import type { Env } from "./types";

const router = new Router();

// Konto
router.get("/v1/auth/prelogin", prelogin);
router.post("/v1/auth/register", register);
router.post("/v1/auth/login", login);
router.post("/v1/auth/logout", logout, { auth: true });
router.post("/v1/auth/password", changePassword, { auth: true });
router.post("/v1/auth/recover", recover);
router.post("/v1/auth/recover/complete", recoverComplete);
router.get("/v1/devices", listDevicesHandler, { auth: true });
router.delete("/v1/devices/:id", deleteDeviceHandler, { auth: true });

// Sync
router.post("/v1/sync/push", push, { auth: true });
router.get("/v1/sync/pull", pull, { auth: true });
router.get("/v1/sync/status", status, { auth: true });

// Anhänge
router.put("/v1/blobs/:id", putBlob, { auth: true });
router.get("/v1/blobs/:id", getBlob, { auth: true });
router.delete("/v1/blobs/:id", deleteBlob, { auth: true });

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    return router.handle(request, env, ctx);
  },
};
