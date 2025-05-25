// ─────────────────────────────────────── admin/documents ────────────────────
'use client'

import { useEffect, useState } from 'react'
import Link                     from 'next/link'
import { Pencil, Trash }        from 'lucide-react'

import { api }                  from '@/lib/api'
import { useToast }             from '@/hooks/use-toast'

import {
    Card, CardHeader, CardTitle, CardContent,
} from '@/components/ui/card'
import { Button }               from '@/components/ui/button'
import {
    AlertDialog, AlertDialogTrigger, AlertDialogContent,
    AlertDialogHeader, AlertDialogTitle, AlertDialogDescription,
    AlertDialogCancel, AlertDialogAction,
} from '@/components/ui/alert-dialog'
import {
    Tooltip, TooltipTrigger, TooltipContent, TooltipProvider,
} from '@/components/ui/tooltip'
import DataTable, { Column }    from '@/components/DataTable'

/* -------------------------------------------------------------------------- */
/* types                                                                      */
/* -------------------------------------------------------------------------- */
type Document = {
    id          : number
    filename    : string
    uploaded_by?: string
}

/* tiny helper for re-usable icon tooltips */
function IconBtnTooltip(props:{ label:string; children:React.ReactNode }) {
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
export default function AdminDocuments() {
    const { toast }  = useToast()
    const [rows, setRows] = useState<Document[]>([])
    const [busy, setBusy] = useState(false)

    async function refresh() {
        setBusy(true)
        try   { setRows(await api<Document[]>('/api/documents')) }
        catch (e){ toast({ variant:'destructive', title:'Failed to load documents', description:String(e) }) }
        finally { setBusy(false) }
    }
    useEffect(()=>{ refresh() }, [])

    const cols: Column<Document>[] = [
        { key:'id',          label:'ID' },
        { key:'filename',    label:'Filename', className:'truncate max-w-[12rem]' },
        { key:'uploaded_by', label:'Uploader', className:'capitalize', render:v=>v ?? '–' },
        {
            key:'actions' as unknown as keyof Document, label:'', render:(_, row)=>(
                <div className="flex gap-2 justify-end">
                    {/* edit (rename) */}
                    <IconBtnTooltip label="Edit">
                        <Link href={`/documents/${row.id}`} aria-label="Edit">
                            <Button size="icon" variant="ghost">
                                <Pencil className="size-4"/>
                            </Button>
                        </Link>
                    </IconBtnTooltip>

                    {/* delete */}
                    <DeleteBtn doc={row} onDone={refresh}/>
                </div>
            ),
        },
    ]

    return (
        <Card>
            <CardHeader className="flex-row items-center justify-between">
                <div>
                    <CardTitle>Documents</CardTitle>
                    <p className="text-sm text-muted-foreground">
                        Uploaded files currently stored in the knowledge-base.
                    </p>
                </div>

                <Link href="/upload"><Button size="sm">Upload more</Button></Link>
            </CardHeader>

            <CardContent>
                <DataTable<Document> data={rows} columns={cols}/>
                {busy && <p className="mt-4 text-sm text-muted-foreground">Loading…</p>}
            </CardContent>
        </Card>
    )
}

/* ═════════════ Delete ═════════════ */

function DeleteBtn({ doc, onDone }:{
    doc:Document; onDone:()=>void
}) {
    const { toast } = useToast()
    const [busy,setBusy] = useState(false)

    async function handleDelete(close:()=>void){
        setBusy(true)
        try {
            await api(`/api/documents/${doc.id}`, { method:'DELETE' })
            toast({ title:`Deleted ${doc.filename}` })
            close(); onDone()
        } catch (e){
            toast({ variant:'destructive', title:'Delete failed', description:String(e) })
        } finally { setBusy(false) }
    }

    return (
        <AlertDialog>
            <IconBtnTooltip label="Delete">
                <AlertDialogTrigger asChild>
                    <Button size="icon" variant="ghost"><Trash className="size-4"/></Button>
                </AlertDialogTrigger>
            </IconBtnTooltip>

            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Delete {doc.filename}?</AlertDialogTitle>
                    <AlertDialogDescription>This will remove the file and its embeddings.</AlertDialogDescription>
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
