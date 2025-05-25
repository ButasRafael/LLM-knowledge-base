'use client'

import { useState, useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

import { api }      from '@/lib/api'
import { useToast } from '@/hooks/use-toast'

import {
    Card, CardHeader, CardTitle, CardContent, CardFooter,
} from '@/components/ui/card'
import { Button }  from '@/components/ui/button'
import { Input }   from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'

import { Loader2, Plus } from 'lucide-react'

type Conv = { id: number; title: string }
type Msg  = { id: number; sender: 'user' | 'assistant'; content: string }

export default function ChatPage() {
    const { toast }        = useToast()
    const [convs, setConvs] = useState<Conv[]>([])
    const [sel,   setSel]   = useState<Conv | null>(null)
    const [msgs,  setMsgs]  = useState<Msg[]>([])
    const [prompt, setPr]   = useState('')
    const [busy,   setBusy] = useState(false)
    const scEnd             = useRef<HTMLDivElement>(null)

    /* ─── helpers ────────────────────────────────────────────── */

    /** Pull the sidebar list (and auto‐select the newest chat). */
    const loadList = async () => {
        try {
            const { conversations } = await api<{ conversations: Conv[] }>('/api/chat/conversations')
            setConvs(conversations)
            if (!sel && conversations.length) {
                void select(conversations[0])
            }
        } catch (err) {
            toast({ variant:'destructive', title:'Failed to load chats', description:String(err) })
        }
    }

    /** Load messages for one conversation. */
    const select = async (c: Conv) => {
        setSel(c)
        setMsgs([])
        try {
            const data = await api<Msg[]>(`/api/chat/conversations/${c.id}/messages`)
            // API returns newest‐first; we want oldest‐first in the UI
            setMsgs(data.reverse())
        } catch (err) {
            toast({ variant:'destructive', title:'Failed to load messages', description:String(err) })
        }
    }

    const send = async () => {
        if (!sel || !prompt.trim()) return;

        const userMsg = prompt.trim();
        setMsgs(m => [...m, { id: Date.now(), sender: 'user', content: userMsg }]);
        setPr('');
        setBusy(true);

        // fire off the POST, we don’t expect the answer in the response
        await api(`/api/chat/conversations/${sel.id}/messages`, {
            method: 'POST',
            body: { prompt: userMsg },
        });

        // now poll every second until we see a new assistant message
        const interval = setInterval(async () => {
            const all = await api<Msg[]>(`/api/chat/conversations/${sel.id}/messages`);
            const rev = all.reverse();
            // if our UI has fewer messages than the server:
            if (rev.length > msgs.length) {
                setMsgs(rev);
                clearInterval(interval);
                setBusy(false);
            }
        }, 1_000);
    };

    useEffect(() => {
        scEnd.current?.scrollIntoView({ behavior: 'smooth' })
    }, [msgs])

    // initial load
    useEffect(() => { void loadList() }, [])

    /* ─── render ─────────────────────────────────────────────── */

    return (
        <div className="flex h-[calc(100vh-5rem)] gap-4">
            {/* ─── sidebar – conversations ────────────────────────── */}
            <Card className="flex w-56 flex-col">
                <CardHeader className="flex-row items-center justify-between">
                    <CardTitle className="text-base">Chats</CardTitle>
                    <Button
                        size="icon"
                        variant="ghost"
                        onClick={async () => {
                            const title = window.prompt('Give your new chat a name:')
                            if (title == null) return
                            try {
                                const conv = await api<Conv>(
                                    '/api/chat/conversations',
                                    { method: 'POST', body: { title } }
                                )
                                await loadList()
                                void select(conv)
                            } catch (err) {
                                toast({ variant:'destructive', title:'Couldn’t create chat', description:String(err) })
                            }
                        }}
                    >
                        <Plus className="size-4" />
                    </Button>
                </CardHeader>
                <ScrollArea className="flex-1">
                    <ul className="space-y-1 px-2 pb-2">
                        {convs.map(c => (
                            <li key={c.id}>
                                <button
                                    onClick={() => void select(c)}
                                    className={`w-full rounded px-2 py-1 text-left hover:bg-muted
                    ${sel?.id === c.id ? 'bg-muted font-medium' : ''}`}
                                >
                                    {c.title}
                                </button>
                            </li>
                        ))}
                    </ul>
                </ScrollArea>
            </Card>

            {/* ─── main panel – messages ──────────────────────────── */}
            <Card className="flex flex-1 flex-col">
                <CardHeader>
                    <CardTitle>{sel?.title ?? 'Select a chat'}</CardTitle>
                </CardHeader>

                <CardContent className="flex flex-1 flex-col overflow-y-auto p-4">
                    <div className="flex flex-col gap-3">
                        {msgs.map(m => (
                            <div
                                key={m.id}
                                className={`max-w-[75%] rounded-lg p-3 text-sm
                  ${m.sender === 'user' ? 'self-end bg-primary/10' : 'self-start bg-muted'}`}
                            >
                                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                                    {m.content}
                                </ReactMarkdown>
                            </div>
                        ))}
                        <div ref={scEnd} />
                    </div>
                </CardContent>

                <CardFooter>
                    <form
                        onSubmit={e => { e.preventDefault(); send() }}
                        className="flex w-full items-center gap-3"
                    >

                        <Input
                            className="flex-1"
                            value={prompt}
                            onChange={e => setPr(e.target.value)}
                            disabled={busy || !sel}
                        />
                        <Button disabled={busy || !prompt.trim() || !sel}>
                            {busy
                                ? <Loader2 className="size-4 animate-spin" />
                                : 'Send'}
                        </Button>
                    </form>
                </CardFooter>
            </Card>
        </div>
    )
}
