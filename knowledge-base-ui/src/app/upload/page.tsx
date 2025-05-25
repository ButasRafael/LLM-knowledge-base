// ─────────────────────────────────────────────────────────────────────────────
// src/app/upload/page.tsx
// ─────────────────────────────────────────────────────────────────────────────
'use client'

import {
    Card, CardContent, CardDescription, CardHeader, CardTitle,
} from '@/components/ui/card'
import { Button }   from '@/components/ui/button'
import { Input }    from '@/components/ui/input'
import { useToast } from '@/hooks/use-toast'
import { useRouter } from 'next/navigation'
import {
    FileText, UploadCloud, X as RemoveIcon,
} from 'lucide-react'
import { useCallback, useRef, useState, DragEvent } from 'react'

const MAX_FILE_SIZE = 50 * 1024 * 1024                     // 50 MB
const ALLOWED_TYPES = [
    'application/pdf',
    'text/plain',
    'text/markdown',
]

type UiFile = {
    file: File
    id  : string          // stable key for React lists
}

function humanSize (bytes: number) {
    return bytes > 1024*1024
        ? `${(bytes/1024/1024).toFixed(1)} MB`
        : `${(bytes/1024).toFixed(0)} KB`
}

export default function UploadPage() {
    const [files,    setFiles   ] = useState<UiFile[]>([])
    const [busy,     setBusy    ] = useState(false)
    const router                 = useRouter()
    const { toast }              = useToast()
    const inputRef               = useRef<HTMLInputElement>(null)

    /* ───────────── helpers ───────────── */

    const addFiles = useCallback((list: FileList) => {
        const next: UiFile[] = []
        for (const f of Array.from(list)) {
            if (!ALLOWED_TYPES.includes(f.type)) {
                toast({ variant:'destructive',
                    title:'Unsupported file type',
                    description:`${f.name} – ${f.type || 'unknown'}` })
                continue
            }
            if (f.size > MAX_FILE_SIZE) {
                toast({ variant:'destructive',
                    title:'File too large',
                    description:`${f.name} is ${humanSize(f.size)} (limit 50 MB)` })
                continue
            }
            next.push({ file:f, id:crypto.randomUUID() })
        }
        if (next.length) setFiles(prev => [...prev, ...next])
    }, [toast])

    /* drop handler ---------------------------------------------------- */
    const handleDrop = (e: DragEvent<HTMLDivElement>) => {
        e.preventDefault()
        addFiles(e.dataTransfer.files)
    }

    /* browse handler -------------------------------------------------- */
    const handleBrowse = () => inputRef.current?.click()
    const handleInput  = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files) addFiles(e.target.files)
        e.target.value = ''                               // reset <input>
    }

    /* remove single file --------------------------------------------- */
    const removeFile = (id: string) =>
        setFiles(prev => prev.filter(f => f.id !== id))

    /* upload ---------------------------------------------------------- */
    async function doUpload() {
        if (!files.length) return
        setBusy(true)

        const form = new FormData()
        files.forEach(f => form.append('file', f.file))

        try {
            const res = await fetch('http://localhost:8000/api/documents/upload', {
                method      : 'POST',
                body        : form,
                credentials : 'include',          // keep auth-token cookie
            })

            if (!res.ok)
                throw new Error(await res.text())

            toast({ title:'Upload complete', description:`${files.length} file(s) processed` })
            router.push('/documents')
            router.refresh()
        } catch (err) {
            toast({ variant:'destructive', title:'Upload failed', description:String(err) })
        } finally {
            setBusy(false)
            setFiles([])
        }
    }

    /* ───────────── UI ───────────── */

    return (
        <Card className="mx-auto w-full max-w-2xl">
            <CardHeader>
                <CardTitle>Upload documents</CardTitle>
                <CardDescription>
                    PDF, Markdown or plain-text – up to 50 MB each. They&rsquo;ll be
                    embedded and indexed automatically.
                </CardDescription>
            </CardHeader>

            <CardContent className="space-y-6">
                {/* drag-n-drop area */}
                <div
                    onDragOver={e => e.preventDefault()}
                    onDrop={handleDrop}
                    className="flex flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed border-muted p-8 text-muted-foreground
                     hover:bg-muted/40 transition-colors"
                >
                    <UploadCloud className="size-8" />
                    <p className="text-sm">Drag files here or</p>
                    <Button size="sm" variant="secondary" onClick={handleBrowse}>
                        Browse…
                    </Button>

                    {/* hidden real input */}
                    <Input
                        ref={inputRef}
                        type="file"
                        multiple
                        className="hidden"
                        accept={ALLOWED_TYPES.join(',')}
                        onChange={handleInput}
                    />
                </div>

                {/* selected files list */}
                {files.length > 0 && (
                    <ul className="space-y-2">
                        {files.map(({ id, file }) => (
                            <li key={id} className="flex items-center justify-between rounded-md bg-muted/40 px-3 py-2">
                                <div className="flex items-center gap-2">
                                    <FileText className="size-4" />
                                    <span className="truncate max-w-[14rem]">{file.name}</span>
                                    <span className="text-xs text-muted-foreground">({humanSize(file.size)})</span>
                                </div>
                                <Button size="icon" variant="ghost" onClick={() => removeFile(id)}>
                                    <RemoveIcon className="size-4" />
                                </Button>
                            </li>
                        ))}
                    </ul>
                )}

                {/* actions */}
                <div className="flex justify-end gap-2">
                    <Button variant="ghost" onClick={() => setFiles([])} disabled={busy || !files.length}>
                        Clear
                    </Button>
                    <Button onClick={doUpload} disabled={busy || !files.length}>
                        {busy ? 'Uploading…' : `Upload ${files.length || ''}`}
                    </Button>
                </div>
            </CardContent>
        </Card>
    )
}
