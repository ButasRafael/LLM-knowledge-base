import './globals.css'
import type { Metadata } from 'next'
import { Geist, Geist_Mono } from 'next/font/google'
import Sidebar from '@/components/Sidebar'
import { TopBar } from '@/components/TopBar'
import { getCtxFromCookie } from '@/lib/auth'

/* ────────── fonts ────────── */
const geistSans = Geist({
    variable: '--font-geist-sans',
    subsets: ['latin'],
})
const geistMono = Geist_Mono({
    variable: '--font-geist-mono',
    subsets: ['latin'],
})

/* ────────── <head> meta ────────── */
export const metadata: Metadata = {
    title: 'Knowledge-Base',
    description: 'Rust + Next.js playground',
}

/* ────────── layout component ────────── */
export default async function RootLayout({
                                             children,
                                         }: {
    children: React.ReactNode
}) {
    // Parse the `auth-token` cookie server-side
    const ctx = await getCtxFromCookie()

    // When not logged-in just render the page (login / register)
    if (!ctx) {
        return (
            <html lang="en" suppressHydrationWarning>
            <body
                className={`${geistSans.variable} ${geistMono.variable} antialiased bg-background text-foreground`}
            >
            {children}
            </body>
            </html>
        )
    }

    /* ─── Authenticated chrome (sidebar + topbar) ─── */
    return (
        <html lang="en" suppressHydrationWarning>
        <body
            className={`${geistSans.variable} ${geistMono.variable} antialiased min-h-screen flex bg-background text-foreground`}
        >
        {/* permanent sidebar ≥ lg, hidden on mobile */}
        <Sidebar className="hidden lg:block" />

        <div className="flex flex-1 flex-col">
            <TopBar ctx={ctx} />

            <main className="flex-1 overflow-auto p-4">{children}</main>
        </div>
        </body>
        </html>
    )
}
