'use client'

import { useEffect, useState } from 'react'
import Link                     from 'next/link'
import { api }                  from '@/lib/api'
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
} from '@/components/ui/card'
import DataTable, { type Column } from '@/components/DataTable'

type DocRow = { id: number; filename: string; uploaded_by?: string }

export default function Dashboard() {
  const [recent, setRecent] = useState<DocRow[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    setLoading(true)
    api<DocRow[]>('/api/documents')
        .then(d => setRecent(d))
        .catch(console.error)
        .finally(() => setLoading(false))
  }, [])

  const columns: Column<DocRow>[] = [
    { key: 'id',          label: 'ID' },
    { key: 'filename',    label: 'Filename',   className: 'truncate max-w-[12rem]' },
    { key: 'uploaded_by', label: 'Uploaded by', render: v => v ?? '–', className: 'capitalize' },
  ]

  return (
      <div className="grid gap-6 lg:grid-cols-3">
        {/* shortcuts */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Welcome 👋</CardTitle>
            <CardDescription>Jump straight in with these shortcuts.</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <Link href="/upload"        className="hover:underline">Upload documents</Link>
            <Link href="/chat"          className="hover:underline">Chat</Link>
            <Link href="/tasks"         className="hover:underline">Tasks</Link>
            <Link href="/users"   className="hover:underline">Users</Link>
            <Link href="/documents" className="hover:underline">Documents</Link>
            <Link href="/metrics" className="hover:underline">Metrics</Link>
            <Link href="/profile"       className="hover:underline text-primary">Profile</Link>
            <Link href="/login"         className="hover:underline text-primary">Sign in</Link>
          </CardContent>
        </Card>

        {/* live recent uploads */}
        <Card>
          <CardHeader>
            <CardTitle>Recent uploads</CardTitle>
          </CardHeader>
          <CardContent>
            {loading
                ? <p className="text-sm text-muted-foreground">Loading…</p>
                : <DataTable<DocRow> data={recent} columns={columns} />
            }
          </CardContent>
        </Card>
      </div>
  )
}
