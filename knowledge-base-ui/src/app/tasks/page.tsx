'use client'

import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import { useToast } from '@/hooks/use-toast'
import {
    Card,
    CardHeader,
    CardTitle,
    CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import {
    AlertDialog,
    AlertDialogTrigger,
    AlertDialogContent,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogDescription,
    AlertDialogCancel,
    AlertDialogAction,
} from '@/components/ui/alert-dialog'
import {
    Tooltip,
    TooltipTrigger,
    TooltipContent,
    TooltipProvider,
} from '@/components/ui/tooltip'
import DataTable, { Column } from '@/components/DataTable'
import { Plus, Trash } from 'lucide-react'

type Task = {
    id: number
    title: string
}

type TaskWithActions = Task & { actions?: never }

function IconBtnTooltip(props: { label: string; children: React.ReactNode }) {
    return (
        <TooltipProvider delayDuration={300}>
            <Tooltip>
                <TooltipTrigger asChild>{props.children}</TooltipTrigger>
                <TooltipContent>{props.label}</TooltipContent>
            </Tooltip>
        </TooltipProvider>
    )
}

export default function AdminTasks() {
    const { toast } = useToast()
    const [rows, setRows] = useState<Task[]>([])
    const [busy, setBusy] = useState(false)

    async function refresh() {
        setBusy(true)
        try {
            setRows(await api<Task[]>('/api/tasks'))
        } catch (e) {
            toast({
                variant: 'destructive',
                title: 'Failed to load tasks',
                description: String(e),
            })
        } finally {
            setBusy(false)
        }
    }

    useEffect(() => {
        void refresh()
    }, [])

    const cols: Column<TaskWithActions>[] = [
        { key: 'id', label: 'ID' },
        { key: 'title', label: 'Title', className: 'truncate max-w-[12rem]' },
        {
            key: 'actions',
            label: '',
            render: (_v, row) => (
                <div className="flex gap-2 justify-end">
                    <DeleteBtn task={row} onDone={refresh} />
                </div>
            ),
        },
    ]

    return (
        <Card>
            <CardHeader className="flex-row items-center justify-between">
                <div>
                    <CardTitle>Tasks</CardTitle>
                    <p className="text-sm text-muted-foreground">
                        Create, view or delete tasks.
                    </p>
                </div>
                <CreateBtn onDone={refresh} />
            </CardHeader>

            <CardContent>
                <DataTable<Task> data={rows} columns={cols} />
                {busy && (
                    <p className="mt-4 text-sm text-muted-foreground">Loading…</p>
                )}
            </CardContent>
        </Card>
    )
}

/* ═════════════ Create ═════════════ */
function CreateBtn({ onDone }: { onDone: () => void }) {
    const { toast } = useToast()
    const [open, setOpen] = useState(false)
    const [title, setTitle] = useState('')
    const [busy, setBusy] = useState(false)

    async function handleSave() {
        setBusy(true)
        try {
            await api('/api/tasks', { method: 'POST', body: { title } })
            toast({ title: 'Task created' })
            setOpen(false)
            setTitle('')
            onDone()
        } catch (e) {
            toast({
                variant: 'destructive',
                title: 'Create failed',
                description: String(e),
            })
        } finally {
            setBusy(false)
        }
    }

    return (
        <AlertDialog open={open} onOpenChange={setOpen}>
            <IconBtnTooltip label="New Task">
                <AlertDialogTrigger asChild>
                    <Button size="sm" onClick={() => setOpen(true)}>
                        <Plus className="mr-1 size-4" /> New Task
                    </Button>
                </AlertDialogTrigger>
            </IconBtnTooltip>

            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Create Task</AlertDialogTitle>
                    <AlertDialogDescription>
                        Enter a title for your new task.
                    </AlertDialogDescription>
                </AlertDialogHeader>

                <div className="mt-4 grid gap-2">
                    <label htmlFor="task-title" className="text-sm font-medium">
                        Title
                    </label>
                    <input
                        id="task-title"
                        className="w-full rounded border px-2 py-1"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                    />
                </div>

                <div className="flex justify-end gap-2 pt-4">
                    <AlertDialogCancel asChild>
                        <Button variant="ghost">Cancel</Button>
                    </AlertDialogCancel>
                    <AlertDialogAction asChild>
                        <Button
                            onClick={handleSave}
                            disabled={busy || !title.trim()}
                        >
                            {busy ? 'Saving…' : 'Save'}
                        </Button>
                    </AlertDialogAction>
                </div>
            </AlertDialogContent>
        </AlertDialog>
    )
}

/* ═════════════ Delete ═════════════ */
function DeleteBtn({
                       task,
                       onDone,
                   }: {
    task: Task
    onDone: () => void
}) {
    const { toast } = useToast()
    const [busy, setBusy] = useState(false)

    async function handleDelete(close: () => void) {
        setBusy(true)
        try {
            await api(`/api/tasks/${task.id}`, { method: 'DELETE' })
            toast({ title: `Deleted "${task.title}"` })
            close()
            onDone()
        } catch (e) {
            toast({
                variant: 'destructive',
                title: 'Delete failed',
                description: String(e),
            })
        } finally {
            setBusy(false)
        }
    }

    return (
        <AlertDialog>
            <IconBtnTooltip label="Delete">
                <AlertDialogTrigger asChild>
                    <Button size="icon" variant="ghost">
                        <Trash className="size-4" />
                    </Button>
                </AlertDialogTrigger>
            </IconBtnTooltip>

            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>
                        Delete &quot;{task.title}&quot;?
                    </AlertDialogTitle>
                    <AlertDialogDescription>
                        This action cannot be undone.
                    </AlertDialogDescription>
                </AlertDialogHeader>

                <div className="flex justify-end gap-2 pt-4">
                    <AlertDialogCancel asChild>
                        <Button variant="ghost">Cancel</Button>
                    </AlertDialogCancel>
                    <AlertDialogAction asChild>
                        <Button
                            variant="destructive"
                            disabled={busy}
                            onClick={(e) => {
                                e.preventDefault()
                                handleDelete(() =>
                                    (e.currentTarget as HTMLElement).click()
                                )
                            }}
                        >
                            {busy ? 'Deleting…' : 'Delete'}
                        </Button>
                    </AlertDialogAction>
                </div>
            </AlertDialogContent>
        </AlertDialog>
    )
}
