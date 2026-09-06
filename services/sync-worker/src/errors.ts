/**
 * Uniform error handling: `{ error, message }` JSON bodies with the status
 * codes required by SYNC_PROTOCOL.md / the task spec (400, 401, 403, 404,
 * 409, 413, 429).
 *
 * IMPORTANT: never put request bodies, tokens, or other secrets into an
 * ApiError message -- these strings can end up in responses and, on the
 * fallback path, in server logs.
 */

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

export function badRequest(message: string, code = "bad_request"): ApiError {
  return new ApiError(400, code, message);
}

export function unauthorized(message: string, code = "unauthorized"): ApiError {
  return new ApiError(401, code, message);
}

export function forbidden(message: string, code = "forbidden"): ApiError {
  return new ApiError(403, code, message);
}

export function notFound(message: string, code = "not_found"): ApiError {
  return new ApiError(404, code, message);
}

export function conflict(message: string, code = "conflict"): ApiError {
  return new ApiError(409, code, message);
}

export function payloadTooLarge(message: string, code = "payload_too_large"): ApiError {
  return new ApiError(413, code, message);
}

export function tooManyRequests(message: string, code = "too_many_requests"): ApiError {
  return new ApiError(429, code, message);
}

export function json(body: unknown, status = 200, headers?: Record<string, string>): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json; charset=utf-8", ...headers },
  });
}

export function noContent(): Response {
  return new Response(null, { status: 204 });
}

/** Turns any thrown value into a JSON error Response. Never echoes raw bodies or tokens. */
export function errorResponse(err: unknown): Response {
  if (err instanceof ApiError) {
    return json({ error: err.code, message: err.message }, err.status);
  }
  // Do not log the error object itself if it might embed request data; a
  // bare marker is enough for the Worker's own observability (see
  // wrangler.toml [observability]).
  console.error("unhandled_error");
  return json({ error: "internal", message: "internal server error" }, 500);
}
