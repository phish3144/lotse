/**
 * /v1/blobs/:id -- SYNC_PROTOCOL.md section 4 "Anhänge". Blob content is raw
 * client-side ciphertext; the service only ever learns account_id, id and
 * size (THREAT_MODEL.md section 3).
 */
import { deleteBlobMeta, upsertBlobMeta } from "./db";
import { badRequest, json, noContent, notFound, payloadTooLarge, unauthorized } from "./errors";
import type { RequestContext } from "./types";

const MAX_BLOB_BYTES = 100 * 1024 * 1024;

function blobKey(accountId: string, id: string): string {
  return `${accountId}/${id}`;
}

function requireBlobId(params: Record<string, string>): string {
  const id = params.id;
  // Blob IDs are ULIDs (SYNC_PROTOCOL.md section 4), but the one rule the
  // server truly must enforce is "no path separator", since the id is
  // concatenated into the R2 key below.
  if (!id || id.length === 0 || id.includes("/")) {
    throw badRequest("invalid blob id", "invalid_field");
  }
  return id;
}

export async function putBlob({ request, env, auth, params }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const id = requireBlobId(params);

  const contentLengthHeader = request.headers.get("content-length");
  if (!contentLengthHeader) {
    throw badRequest("Content-Length header is required", "missing_content_length");
  }
  const contentLength = Number(contentLengthHeader);
  if (!Number.isInteger(contentLength) || contentLength < 0) {
    throw badRequest("invalid Content-Length header", "invalid_content_length");
  }
  if (contentLength > MAX_BLOB_BYTES) {
    throw payloadTooLarge("blob exceeds 100 MiB limit", "blob_too_large");
  }
  if (!request.body) {
    throw badRequest("request body is required", "missing_body");
  }

  const key = blobKey(auth.accountId, id);
  const object = await env.BLOBS.put(key, request.body, {
    httpMetadata: { contentType: "application/octet-stream" },
  });
  const size = object?.size ?? contentLength;

  await upsertBlobMeta(env.DB, auth.accountId, id, size, Date.now());

  return json({ id, size }, 201);
}

export async function getBlob({ env, auth, params }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const id = requireBlobId(params);
  const key = blobKey(auth.accountId, id);

  const object = await env.BLOBS.get(key);
  if (!object) {
    throw notFound("no such blob");
  }
  return new Response(object.body, {
    status: 200,
    headers: {
      "content-type": "application/octet-stream",
      "content-length": String(object.size),
    },
  });
}

export async function deleteBlob({ env, auth, params }: RequestContext): Promise<Response> {
  if (!auth) throw unauthorized("missing session");
  const id = requireBlobId(params);
  const key = blobKey(auth.accountId, id);

  await env.BLOBS.delete(key);
  await deleteBlobMeta(env.DB, auth.accountId, id);
  return noContent();
}
