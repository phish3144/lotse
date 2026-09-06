/**
 * A tiny hand-rolled router -- no hono, no itty-router. Supports `:param`
 * path segments and an `auth: true` flag that runs the bearer-token
 * middleware (src/auth.ts#requireAuth) before the handler.
 */
import { requireAuth } from "./auth";
import { errorResponse, json } from "./errors";
import type { Env, RequestContext } from "./types";

export type Handler = (ctx: RequestContext) => Promise<Response>;

interface Route {
  method: string;
  pattern: RegExp;
  keys: string[];
  handler: Handler;
  auth: boolean;
}

function compilePath(path: string): { pattern: RegExp; keys: string[] } {
  const keys: string[] = [];
  const escaped = path
    .split("/")
    .map((segment) => {
      if (segment.startsWith(":")) {
        keys.push(segment.slice(1));
        return "([^/]+)";
      }
      return segment.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    })
    .join("/");
  return { pattern: new RegExp(`^${escaped}$`), keys };
}

export class Router {
  private routes: Route[] = [];

  add(method: string, path: string, handler: Handler, options: { auth?: boolean } = {}): void {
    const { pattern, keys } = compilePath(path);
    this.routes.push({ method: method.toUpperCase(), pattern, keys, handler, auth: options.auth ?? false });
  }

  get(path: string, handler: Handler, options?: { auth?: boolean }): void {
    this.add("GET", path, handler, options);
  }
  post(path: string, handler: Handler, options?: { auth?: boolean }): void {
    this.add("POST", path, handler, options);
  }
  put(path: string, handler: Handler, options?: { auth?: boolean }): void {
    this.add("PUT", path, handler, options);
  }
  delete(path: string, handler: Handler, options?: { auth?: boolean }): void {
    this.add("DELETE", path, handler, options);
  }

  async handle(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    const method = request.method.toUpperCase();

    let pathMatchedAnyMethod = false;
    for (const route of this.routes) {
      const match = route.pattern.exec(url.pathname);
      if (!match) continue;
      pathMatchedAnyMethod = true;
      if (route.method !== method) continue;

      const params: Record<string, string> = {};
      route.keys.forEach((key, i) => {
        params[key] = decodeURIComponent(match[i + 1] ?? "");
      });

      try {
        const auth = route.auth ? await requireAuth(request, env) : undefined;
        return await route.handler({ request, env, ctx, params, auth });
      } catch (err) {
        return errorResponse(err);
      }
    }

    if (pathMatchedAnyMethod) {
      return json({ error: "method_not_allowed", message: `${method} not supported for this path` }, 405);
    }
    return json({ error: "not_found", message: "no such route" }, 404);
  }
}
