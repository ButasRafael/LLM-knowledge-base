// middleware.ts
import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

function redirectToLogin(req: NextRequest) {
    const url = req.nextUrl.clone()
    url.pathname = '/login'
    // preserve `?from=...` or similar if you like:
    // url.searchParams.set('from', req.nextUrl.pathname)
    return NextResponse.redirect(url)
}

export function middleware(req: NextRequest) {
    const { pathname } = req.nextUrl
    const token = req.cookies.get('auth-token')?.value

    // Helper: is token present, well-formed and unexpired?
    function isValidToken(tok: string): { username: string; exp: Date } | null {
        const parts = tok.split(':')
        if (parts.length < 2) return null
        try {
            const user = Buffer.from(parts[0], 'base64url').toString('utf8')
            const expStr = Buffer.from(parts[1], 'base64url').toString('utf8')
            const exp = new Date(expStr)
            if (isNaN(exp.getTime()) || exp <= new Date()) return null
            return { username: user, exp }
        } catch {
            return null
        }
    }

    // PUBLIC / unguarded paths:
    //   /login, /register, assets, Next internals, etc
    if (
        pathname.startsWith('/login') ||
        pathname.startsWith('/register') ||
        pathname.startsWith('/_next') ||
        pathname.startsWith('/favicon.ico')
    ) {
        return NextResponse.next()
    }

    // Protected **client** routes
    if (
        pathname.startsWith('/chat') ||
        pathname.startsWith('/tasks') ||
        pathname.startsWith('/documents') ||
        pathname.startsWith('/profile') ||
        (pathname.startsWith('/users') && !pathname.startsWith('/users/register'))
    ) {
        if (!token) return redirectToLogin(req)
        const info = isValidToken(token)
        if (!info) return redirectToLogin(req)
        return NextResponse.next()
    }

    // Protected **admin** routes
    if (pathname.startsWith('/admin')) {
        if (!token) return redirectToLogin(req)
        const info = isValidToken(token)
        if (!info) return redirectToLogin(req)
        // simple “role” check by username; replace with real claim once available
        if (!/admin/i.test(info.username)) {
            // optionally redirect to “403” or back to “/”
            return redirectToLogin(req)
        }
        return NextResponse.next()
    }

    // everything else is public
    return NextResponse.next()
}

export const config = {
    // run this middleware for all pages except Next internals and static assets
    matcher: ['/((?!_next|favicon.ico).*)'],
}
