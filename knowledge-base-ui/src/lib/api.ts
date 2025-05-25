// src/lib/api.ts
/**
 * A minimal wrapper around `fetch` that
 *   • automatically hits the Rust backend (BASE)
 *   • always sends/receives JSON
 *   • keeps cookies (so Axum sees your session)
 *
 *  ➜  The only difference from the previous version is the custom
 *      `JsonInit` type, which lets you pass a plain object as `body`
 *      without fighting TypeScript.
 */

export const BASE =
    process.env.NEXT_PUBLIC_API_BASE ?? 'http://localhost:8000'

/** Same as RequestInit but `body` can be *any* serialisable value. */
type JsonInit = Omit<RequestInit, 'body'> & { body?: unknown }

export async function api<T = unknown>(
    path: string,
    opts: JsonInit = {},
): Promise<T> {
    // ─── normalise body ───────────────────────────────────────────────
    const rawBody = opts.body
    const body: BodyInit | null | undefined =
        rawBody &&
        typeof rawBody === 'object' &&
        !(rawBody instanceof FormData) &&
        !(rawBody instanceof Blob) &&
        !(rawBody instanceof ArrayBuffer) &&
        !(rawBody instanceof URLSearchParams)
            ? JSON.stringify(rawBody)
            : (rawBody as BodyInit | null | undefined)

    // ─── perform request ──────────────────────────────────────────────
    const res = await fetch(`${BASE}${path}`, {
        // ‼️ cookies are critical: the server issues / validates `auth-token`
        credentials: 'include',
        ...opts,
        headers: {
            'Content-Type': 'application/json',
            ...(opts.headers ?? {}),
        },
        body,
    })

    if (!res.ok) {
        // our Axum layer returns { error: { type, req_uuid } }
        const err = await res.json().catch(() => ({}))
        throw new Error(err?.error?.type ?? res.statusText)
    }

    // 204 → no body
    return (res.status === 204 ? undefined : res.json()) as Promise<T>
}
