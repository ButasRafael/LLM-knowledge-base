// src/lib/auth.ts
import { cookies } from 'next/headers';

/** Auth-aware context forwarded to Layout, Sidebar, TopBar, etc. */
export type Ctx = {
    username: string;
    isAdmin: boolean;
    exp: Date;
};

/* ---------- tiny helpers ---------- */

/** URL-safe base-64 decode (no external lib needed) */
function b64uDecode(str: string): string {
    // Node ≥18 supports the “base64url” flavour natively
    return Buffer.from(str, 'base64url').toString('utf8');
}

/** True if date string is valid *and* in the future */
function isFutureISO(dateStr: string): boolean {
    const d = new Date(dateStr);
    return !isNaN(d.getTime()) && d > new Date();
}

/* ---------- public API ---------- */

/**
 * Safely parse the Rust `auth-token` cookie.
 * Returns `null` if missing / expired / malformed.
 *
 * ⚠️ Call this **only in Server Components / Route Handlers** –
 * `next/headers` is not available in the browser.
 */
export async function getCtxFromCookie(): Promise<Ctx | null> {
    // Edge runtime: cookies() is async, so we await it.
    const cookieStore = await cookies();
    const raw = cookieStore.get('auth-token')?.value;
    if (!raw) return null;

    try {
        // Format produced by the Rust backend:
        //   base64u(identifier):base64u(expiration):signature
        const [identifierEnc, expirationEnc] = raw.split(':');
        const username = b64uDecode(identifierEnc);
        const expStr = b64uDecode(expirationEnc);

        if (!isFutureISO(expStr)) return null;

        // super lightweight: "admin" anywhere in the name ⇒ admin role
        const isAdmin = /admin/i.test(username);

        return { username, isAdmin, exp: new Date(expStr) };
    } catch {
        /* malformed cookie */
        return null;
    }
}
