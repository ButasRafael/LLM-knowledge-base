'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { api } from '@/lib/api'
import {
    Card,
    CardHeader,
    CardContent,
    CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { useToast } from '@/hooks/use-toast'

function toMessage(e: unknown): string {
    return e instanceof Error ? e.message : String(e)
}

export default function RegisterPage() {
    const router = useRouter()
    const { toast } = useToast()

    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [loading, setLoading] = useState(false)

    async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault()
        setLoading(true)

        try {
            // create user
            await api('/register', {
                method: 'POST',
                body: { username, pwd_clear: password },
            })
            // auto-login
            await api('/api/login', {
                method: 'POST',
                body: { username, password },
            })
            router.replace('/')
            router.refresh()
        } catch (err) {
            toast({
                variant: 'destructive',
                title: 'Couldn’t register',
                description: toMessage(err),
            })
        } finally {
            setLoading(false)
        }
    }

    return (
        <main className="flex min-h-screen items-center justify-center p-4">
            <Card className="w-full max-w-sm">
                <CardHeader>
                    <CardTitle>Create account</CardTitle>
                </CardHeader>

                <CardContent>
                    <form onSubmit={handleSubmit} className="grid gap-4">
                        <div className="grid gap-2">
                            <Label htmlFor="username">Username</Label>
                            <Input
                                id="username"
                                value={username}
                                onChange={(e) => setUsername(e.target.value)}
                                required
                            />
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="password">Password</Label>
                            <Input
                                id="password"
                                type="password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                required
                            />
                        </div>

                        <Button type="submit" disabled={loading} className="w-full">
                            {loading ? 'Creating…' : 'Register'}
                        </Button>
                    </form>

                    <p className="mt-4 text-sm">
                        Already have an account?{' '}
                        <a href="/login" className="underline">
                            Sign in
                        </a>
                    </p>
                </CardContent>
            </Card>
        </main>
    )
}
