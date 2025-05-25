// ──────────────────────────────────────────
// src/app/profile/page.tsx
// Self-service profile + password change
// ──────────────────────────────────────────
'use client'

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { api }       from '@/lib/api'
import { useToast }  from '@/hooks/use-toast'

import {
    Card, CardHeader, CardTitle, CardContent,
} from '@/components/ui/card'
import { Button }   from '@/components/ui/button'
import { Input }    from '@/components/ui/input'
import { Label }    from '@/components/ui/label'

type Me = { id:number; username:string; role?:string|null }

export default function Profile() {
    const router          = useRouter()
    const { toast }       = useToast()

    const [me,  setMe ] = useState<Me>()
    const [busy, setBusy] = useState(false)

    /* password fields */
    const [oldPwd , setOld ] = useState('')
    const [newPwd , setNew ] = useState('')
    const [newPwd2, setNew2] = useState('')

    /* ───── load “me” once ─────────────────────────────────────────── */
    useEffect(() => {
        api<Me>('/api/users/me')
            .then(setMe)
            .catch(e => toast({
                variant:'destructive',
                title:'Could not load profile',
                description:String(e),
            }))
    }, [toast])

    /* ───── submit password change ─────────────────────────────────── */
    async function handleChange(e: React.FormEvent) {
        e.preventDefault()
        if (!me) return
        if (newPwd !== newPwd2) {
            toast({ variant:'destructive', title:'Passwords don’t match' })
            return
        }

        setBusy(true)
        try {
            await api(`/api/users/${me.id}/password`, {
                method:'PUT',
                body:{ old_password:oldPwd, new_password:newPwd },
            })
            toast({ title:'Password updated ✔︎' })
            setOld(''); setNew(''); setNew2('')
            router.refresh()               // cookie expiry refreshed by mw
        } catch (err) {
            toast({ variant:'destructive', title:'Update failed', description:String(err) })
        } finally { setBusy(false) }
    }

    /* ───── UI ─────────────────────────────────────────────────────── */
    return (
        <Card className="max-w-lg mx-auto">
            <CardHeader>
                <CardTitle>Your profile</CardTitle>
            </CardHeader>

            <CardContent className="space-y-8">
                {/* read-only info */}
                <section>
                    <Label className="text-xs uppercase text-muted-foreground">Username</Label>
                    <p className="mt-1 text-lg font-medium">{me?.username ?? '…'}</p>
                    {me?.role && (
                        <>
                            <Label className="mt-4 text-xs uppercase text-muted-foreground">Role</Label>
                            <p className="mt-1 capitalize">{me.role}</p>
                        </>
                    )}
                </section>

                {/* password change */}
                <form onSubmit={handleChange} className="grid gap-4">
                    <h3 className="font-medium">Change password</h3>

                    <div className="grid gap-2">
                        <Label htmlFor="old">Current password</Label>
                        <Input id="old" type="password" value={oldPwd} onChange={e=>setOld(e.target.value)} required />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="new1">New password</Label>
                        <Input id="new1" type="password" value={newPwd} onChange={e=>setNew(e.target.value)} required />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="new2">Repeat new password</Label>
                        <Input id="new2" type="password" value={newPwd2} onChange={e=>setNew2(e.target.value)} required />
                    </div>

                    <div className="flex justify-end">
                        <Button type="submit" disabled={busy}>{busy ? 'Saving…' : 'Update password'}</Button>
                    </div>
                </form>
            </CardContent>
        </Card>
    )
}
