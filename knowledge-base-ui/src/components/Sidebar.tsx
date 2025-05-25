
'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import { Sheet, SheetContent, SheetTrigger, SheetClose } from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Menu } from 'lucide-react'
import { cn } from '@/lib/utils'

type NavItem = { label: string; href: string }

const PRIMARY_NAV: NavItem[] = [
    { label: 'Dashboard', href: '/' },
    { label: 'Chat',      href: '/chat' },
    { label: 'Tasks',     href: '/tasks' },
    { label: 'Documents', href: '/documents' },
]

const ADMIN_NAV: NavItem[] = [
    { label: 'Users',     href: '/users' },
    { label: 'Documents', href: '/documents' },  // duplicate href
    { label: 'Metrics',   href: '/metrics' },
]

export default function Sidebar({ className, mobile = false }: { className?: string; mobile?: boolean }) {
    const pathname = usePathname()
    const [isAdmin, setIsAdmin] = useState(false)

    // fetch /api/users/me once to know if admin
    useEffect(() => {
        api<{ role?: string }>('/api/users/me')
            .then(u => { if (u.role === 'admin') setIsAdmin(true) })
            .catch(() => {})
    }, [])

    // filter out any ADMIN_NAV items that share a href with PRIMARY_NAV
    const extraAdminNav = isAdmin
        ? ADMIN_NAV.filter(a => !PRIMARY_NAV.some(p => p.href === a.href))
        : []

    // mobile hamburger
    if (mobile) {
        return (
            <Sheet>
                <SheetTrigger asChild>
                    <Button variant="ghost" size="icon" aria-label="Open menu"><Menu /></Button>
                </SheetTrigger>
                <SheetContent side="left" className="p-0 w-64">
                    <nav className="h-full flex flex-col gap-2 pt-4">
                        {[...PRIMARY_NAV, ...extraAdminNav].map(item => (
                            <SheetClose asChild key={item.href}>
                                <Link
                                    href={item.href}
                                    className={cn(
                                        'px-4 py-2 text-sm rounded hover:bg-accent/50',
                                        pathname === item.href && 'bg-accent text-accent-foreground'
                                    )}
                                >{item.label}</Link>
                            </SheetClose>
                        ))}
                        <Separator />
                        <Link href="/profile" className="px-4 py-2 text-sm hover:underline">Profile</Link>
                    </nav>
                </SheetContent>
            </Sheet>
        )
    }

    // desktop sidebar
    return (
        <aside className={cn('sticky top-0 h-screen w-64 shrink-0 border-r bg-muted/20 space-y-3 p-4', className)}>
            <h1 className="text-xl font-semibold">Knowledge Base</h1>
            <Separator />
            <nav className="flex flex-col gap-1">
                {[...PRIMARY_NAV, ...extraAdminNav].map(item => (
                    <Link
                        key={item.href}
                        href={item.href}
                        className={cn(
                            'rounded px-3 py-2 text-sm hover:bg-accent/50',
                            pathname === item.href && 'bg-accent text-accent-foreground'
                        )}
                    >{item.label}</Link>
                ))}
                <Separator />
                <Link
                    href="/profile"
                    className={cn(
                        'rounded px-3 py-2 text-sm hover:bg-accent/50',
                        pathname === '/profile' && 'bg-accent text-accent-foreground'
                    )}
                >Profile</Link>
            </nav>
        </aside>
    )
}
