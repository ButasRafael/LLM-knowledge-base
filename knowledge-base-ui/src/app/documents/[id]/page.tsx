// ──────────────────────────────────────────
// src/app/documents/[id]/page.tsx
// Rename / edit document meta
// ──────────────────────────────────────────
'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { api }       from '@/lib/api'
import { useToast }  from '@/hooks/use-toast'

import {
    Card, CardHeader, CardTitle, CardContent,
} from '@/components/ui/card'
import { Button }   from '@/components/ui/button'
import { Input }    from '@/components/ui/input'
import { Label }    from '@/components/ui/label'

type Doc = {
    id       : number
    filename : string
    // 🔸 add extra optional fields if your `DocumentForUpdate`
    //     struct contains more (e.g. description, tags…)
}

export default function DocDetails() {
    const { id }        = useParams<{ id:string }>()
    const router        = useRouter()
    const { toast }     = useToast()

    const [doc,  setDoc ] = useState<Doc>()
    const [name, setName] = useState('')
    const [busy, setBusy] = useState(false)

    /* ───── pull details once ───────────────────────────────────────── */
    useEffect(() => {
        api<Doc>(`/api/documents/${id}`)
            .then(d => { setDoc(d); setName(d.filename) })
            .catch(e => toast({
                variant:'destructive',
                title:'Could not load document',
                description:String(e),
            }))
    }, [id, toast])

    /* ───── save update ─────────────────────────────────────────────── */
    async function handleSave(e: React.FormEvent) {
        e.preventDefault()
        if (!doc) return
        setBusy(true)
        try {
            await api(`/api/documents/${doc.id}`, {
                method:'PUT',
                body:{ filename:name },          // +other editable meta
            })
            toast({ title:'Document updated' })
            router.back()                      // go back to list
        } catch (err) {
            toast({ variant:'destructive', title:'Update failed', description:String(err) })
        } finally { setBusy(false) }
    }

    /* ───── UI ─────────────────────────────────────────────────────── */
    return (
        <Card className="mx-auto w-full max-w-lg">
            <CardHeader>
                <CardTitle>Document details</CardTitle>
            </CardHeader>

            <CardContent>
                {doc ? (
                    <form onSubmit={handleSave} className="grid gap-6">
                        <div className="grid gap-2">
                            <Label>ID</Label>
                            <p className="text-sm text-muted-foreground">{doc.id}</p>
                        </div>

                        {/* rename ------------------------------------------------ */}
                        <div className="grid gap-2">
                            <Label htmlFor="fn">Filename</Label>
                            <Input id="fn" value={name} onChange={e=>setName(e.target.value)} />
                        </div>

                        {/* ⤵️ put more editable fields here                        */}

                        <div className="flex justify-end gap-2">
                            <Button variant="ghost" type="button" onClick={()=>router.back()}>
                                Cancel
                            </Button>
                            <Button disabled={busy || !name.trim()}>
                                {busy ? 'Saving…' : 'Save'}
                            </Button>
                        </div>
                    </form>
                ) : (
                    <p className="text-sm text-muted-foreground">Loading…</p>
                )}
            </CardContent>
        </Card>
    )
}
