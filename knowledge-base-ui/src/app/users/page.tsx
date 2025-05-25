// ───────────────────────────────────────── admin/users ───────────────────────
'use client'

import { useEffect, useState } from 'react'
import { api }                 from '@/lib/api'
import { useToast }            from '@/hooks/use-toast'
import {
    Card, CardHeader, CardTitle, CardContent,
} from '@/components/ui/card'
import { Button }   from '@/components/ui/button'
import { Input }    from '@/components/ui/input'
import { Label }    from '@/components/ui/label'
import {
    Sheet, SheetContent, SheetHeader, SheetTitle, SheetFooter, SheetClose,
} from '@/components/ui/sheet'
import {
    AlertDialog, AlertDialogTrigger, AlertDialogContent,
    AlertDialogHeader, AlertDialogTitle, AlertDialogDescription,
    AlertDialogCancel, AlertDialogAction,
} from '@/components/ui/alert-dialog'
import {
    Tooltip, TooltipContent, TooltipTrigger, TooltipProvider,
} from '@/components/ui/tooltip'
import DataTable, { Column } from '@/components/DataTable'
import { Plus, Pencil, Trash, Eye } from 'lucide-react'

/* -------------------------------------------------------------------------- */
/* types                                                                      */
/* -------------------------------------------------------------------------- */
type User = {
    id       : number
    username : string
    role?    : string | null
}

/* -------------------------------------------------------------------------- */
/* helpers                                                                    */
/* -------------------------------------------------------------------------- */
function IconBtnTooltip(props: { label:string; children:React.ReactNode }) {
    return (
        <TooltipProvider delayDuration={300}>
            <Tooltip>
                <TooltipTrigger asChild>{props.children}</TooltipTrigger>
                <TooltipContent>{props.label}</TooltipContent>
            </Tooltip>
        </TooltipProvider>
    )
}

/* -------------------------------------------------------------------------- */
/* page                                                                       */
/* -------------------------------------------------------------------------- */
export default function AdminUsers() {
    const { toast }       = useToast()
    const [rows, setRows] = useState<User[]>([])
    const [busy, setBusy] = useState(false)

    /* ---------- fetch list ---------- */
    async function refresh() {
        setBusy(true)
        try   { setRows(await api<User[]>('/api/users')) }
        catch (e){ toast({ variant:'destructive', title:'Failed to load users', description:String(e) }) }
        finally { setBusy(false) }
    }
    useEffect(()=>{ refresh() }, [])

    /* ---------- columns ---------- */
    const cols: Column<User>[] = [
        { key:'id',        label:'ID' },
        { key:'username',  label:'Username', className:'font-medium' },
        { key:'role',      label:'Role', render:v=>v ?? '–', className:'capitalize' },
        {
            key:'actions' as unknown as keyof User, label:'', render:(_, row)=>(
                <div className="flex gap-2 justify-end">
                    <PeekBtn   userId={row.id} />        {/* 👁 quick peek */}
                    <EditBtn   user={row} onDone={refresh}/>
                    <DeleteBtn user={row} onDone={refresh}/>
                </div>
            ),
        },
    ]

    return (
        <Card>
            <CardHeader className="flex-row items-center justify-between">
                <div>
                    <CardTitle>Users</CardTitle>
                    <p className="text-sm text-muted-foreground">
                        Create, edit or delete any user account.
                    </p>
                </div>
                <CreateBtn onDone={refresh}/>
            </CardHeader>

            <CardContent>
                <DataTable<User> data={rows} columns={cols}/>
                {busy && <p className="mt-4 text-sm text-muted-foreground">Loading…</p>}
            </CardContent>
        </Card>
    )
}

/* ═════════════ Quick peek ═════════════ */

function PeekBtn({ userId }:{ userId:number }) {
    const { toast }       = useToast()
    const [open,setOpen]  = useState(false)
    const [data,setData]  = useState<unknown>()

    async function load() {
        if (data || !open) return
        try   { setData(await api(`/api/users/${userId}`)) }
        catch (e){ toast({ variant:'destructive', title:'Fetch failed', description:String(e) }) }
    }

    return (
        <Sheet open={open} onOpenChange={o=>{ setOpen(o); if(o) load() }}>
            <IconBtnTooltip label="Details">
                <Button size="icon" variant="ghost" onClick={()=>setOpen(true)}>
                    <Eye className="size-4"/>
                </Button>
            </IconBtnTooltip>

            <SheetContent side="right" className="w-[26rem]">
                <SheetHeader><SheetTitle>User #{userId}</SheetTitle></SheetHeader>

                {data
                    ? <pre className="mt-4 max-h-[80vh] overflow-auto rounded bg-muted p-4 text-xs">
              {JSON.stringify(data,null,2)}
            </pre>
                    : <p className="mt-6 text-sm text-muted-foreground">Loading…</p>}
            </SheetContent>
        </Sheet>
    )
}

/* ═════════════ Create ═════════════ */

function CreateBtn({ onDone }:{ onDone:()=>void }) {
    const { toast }  = useToast()
    const [open,setOpen] = useState(false)
    const [username,setUsername] = useState('')
    const [password,setPassword] = useState('')
    const [role,setRole]         = useState('user')
    const [busy,setBusy]         = useState(false)

    async function handleSave() {
        setBusy(true)
        try {
            await api('/api/users',{ method:'POST', body:{ username, pwd_clear:password, role } })
            toast({ title:'User created' })
            setOpen(false); onDone()
        } catch(e){
            toast({ variant:'destructive', title:'Create failed', description:String(e) })
        } finally { setBusy(false) }
    }

    return (
        <Sheet open={open} onOpenChange={setOpen}>
            <Button size="sm" onClick={()=>setOpen(true)}>
                <Plus className="mr-1 size-4"/> New user
            </Button>

            <SheetContent side="right" className="w-[24rem] space-y-6">
                <SheetHeader><SheetTitle>Create user</SheetTitle></SheetHeader>

                <div className="grid gap-4">
                    <Label htmlFor="u">Username</Label>
                    <Input id="u" value={username} onChange={e=>setUsername(e.target.value)} />

                    <Label htmlFor="p">Password</Label>
                    <Input id="p" type="password" value={password} onChange={e=>setPassword(e.target.value)} />

                    <Label htmlFor="r">Role&nbsp;(user / admin)</Label>
                    <Input id="r" value={role} onChange={e=>setRole(e.target.value)} />
                </div>

                <SheetFooter className={busy ? 'opacity-50 pointer-events-none' : ''}>
                    <SheetClose asChild><Button variant="ghost">Cancel</Button></SheetClose>
                    <Button onClick={handleSave} disabled={busy||!username||!password}>
                        {busy ? 'Saving…' : 'Save'}
                    </Button>
                </SheetFooter>
            </SheetContent>
        </Sheet>
    )
}

/* ═════════════ Edit ═════════════ */

function EditBtn({ user, onDone }:{ user:User; onDone:()=>void }) {
    const { toast }  = useToast()
    const [open,setOpen]   = useState(false)
    const [username,setUsername] = useState(user.username)
    const [role,setRole]         = useState(user.role ?? '')
    const [busy,setBusy]         = useState(false)

    async function handleSave() {
        setBusy(true)
        try {
            await api(`/api/users/${user.id}`,{ method:'PUT', body:{ username, role } })
            toast({ title:'User updated' })
            setOpen(false); onDone()
        } catch(e){
            toast({ variant:'destructive', title:'Update failed', description:String(e) })
        } finally { setBusy(false) }
    }

    return (
        <Sheet open={open} onOpenChange={setOpen}>
            <IconBtnTooltip label="Edit">
                <Button size="icon" variant="ghost" onClick={()=>setOpen(true)}>
                    <Pencil className="size-4"/>
                </Button>
            </IconBtnTooltip>

            <SheetContent side="right" className="w-[24rem] space-y-6">
                <SheetHeader><SheetTitle>Edit user</SheetTitle></SheetHeader>

                <div className="grid gap-4">
                    <Label htmlFor="eu">Username</Label>
                    <Input id="eu" value={username} onChange={e=>setUsername(e.target.value)} />

                    <Label htmlFor="er">Role</Label>
                    <Input id="er" value={role} onChange={e=>setRole(e.target.value)} />
                </div>

                <SheetFooter className={busy ? 'opacity-50 pointer-events-none' : ''}>
                    <SheetClose asChild><Button variant="ghost">Cancel</Button></SheetClose>
                    <Button onClick={handleSave} disabled={busy||!username}>
                        {busy ? 'Saving…' : 'Save'}
                    </Button>
                </SheetFooter>
            </SheetContent>
        </Sheet>
    )
}

/* ═════════════ Delete ═════════════ */

function DeleteBtn({ user, onDone }:{ user:User; onDone:()=>void }) {
    const { toast } = useToast()
    const [busy,setBusy] = useState(false)

    async function handleDelete(close:()=>void){
        setBusy(true)
        try {
            await api(`/api/users/${user.id}`, { method:'DELETE' })
            toast({ title:`Deleted ${user.username}` })
            close(); onDone()
        } catch (e){
            toast({ variant:'destructive', title:'Delete failed', description:String(e) })
        } finally { setBusy(false) }
    }

    return (
        <AlertDialog>
            <IconBtnTooltip label="Delete">
                <AlertDialogTrigger asChild>
                    <Button size="icon" variant="ghost">
                        <Trash className="size-4"/>
                    </Button>
                </AlertDialogTrigger>
            </IconBtnTooltip>

            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete {user.username}?</AlertDialogTitle>
                    <AlertDialogDescription>This action cannot be undone.</AlertDialogDescription>
                </AlertDialogHeader>

                <div className="flex justify-end gap-2 pt-4">
                    <AlertDialogCancel asChild><Button variant="ghost">Cancel</Button></AlertDialogCancel>
                    <AlertDialogAction asChild>
                        <Button variant="destructive" disabled={busy}
                                onClick={(e)=>{ e.preventDefault(); handleDelete(()=> (e.currentTarget as HTMLElement).click()) }}>
                            {busy ? 'Deleting…' : 'Delete'}
                        </Button>
                    </AlertDialogAction>
                </div>
            </AlertDialogContent>
        </AlertDialog>
    )
}
