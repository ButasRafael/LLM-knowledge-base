'use client'

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import Sidebar from '@/components/Sidebar'
import { Button } from '@/components/ui/button'
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip'
import {
    Avatar,
    AvatarFallback,
    AvatarImage,
} from '@/components/ui/avatar'       // ← shadcn avatar
import { LogOut, Moon, Sun } from 'lucide-react'
import { api } from '@/lib/api'
import { Ctx } from '@/lib/auth'

export function TopBar({ ctx }: { ctx: Ctx | null }) {
    const [dark, setDark] = useState(false)
    const router = useRouter()

    /* dark-mode toggle ----------------------------------------------- */
    useEffect(() => {
        document.documentElement.classList.toggle('dark', dark)
    }, [dark])

    /* log-out --------------------------------------------------------- */
    async function handleLogout() {
        await api('/api/logoff', {
            method: 'POST',
            body: JSON.stringify({ logoff: true }), // ⇐ BodyInit now string
        })
        router.refresh()
    }

    return (
        <header className="sticky top-0 z-30 flex h-14 items-center gap-2 border-b bg-background/70 px-4 backdrop-blur">
            {/* hamburger on mobile */}
            <div className="lg:hidden">
                <Sidebar mobile />
            </div>

            {/* placeholder title / breadcrumbs */}
            <h2 className="flex-1 truncate text-lg font-medium">Dashboard</h2>

            {/* right-hand controls */}
            <TooltipProvider>
                {/* theme button */}
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Button
                            variant="ghost"
                            size="icon"
                            aria-label="Toggle theme"
                            onClick={() => setDark((d) => !d)}
                        >
                            {dark ? <Sun className="size-4" /> : <Moon className="size-4" />}
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>{dark ? 'Light mode' : 'Dark mode'}</TooltipContent>
                </Tooltip>

                {/* log-out button (only if logged-in user exists) */}
                {ctx && (
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="ghost"
                                size="icon"
                                aria-label="Log out"
                                onClick={handleLogout}
                            >
                                <LogOut className="size-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>Log out</TooltipContent>
                    </Tooltip>
                )}

                {/* avatar */}
                <Avatar className="ms-1 hidden sm:block">
                    <AvatarImage src="/avatar.png" />
                    <AvatarFallback>{ctx?.username?.[0] ?? '?'}</AvatarFallback>
                </Avatar>
            </TooltipProvider>
        </header>
    )
}
