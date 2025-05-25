'use client'

import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import {
    Card,
    CardHeader,
    CardTitle,
    CardContent,
} from '@/components/ui/card'
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from 'recharts'

type DayCount = { day: string; count: number }

type StatisticsResponse = {
    total_users: number
    total_tasks: number
    total_documents: number
    total_conversations: number
    total_messages: number
    avg_messages_per_conversation: number

    total_pipeline_runs: number
    avg_pipeline_duration_ms: number

    avg_tasks_per_user: number
    avg_docs_per_user: number

    messages_last_7_days: DayCount[]
}

export default function MetricsPage() {
    const [stats, setStats] = useState<StatisticsResponse | null>(null)
    const [loading, setLoading] = useState(true)

    useEffect(() => {
        api<StatisticsResponse>('/admin/statistics')
            .then((d) => setStats(d))
            .catch(console.error)
            .finally(() => setLoading(false))
    }, [])

    if (loading || !stats) {
        return <p className="p-4 text-center">Loading…</p>
    }

    return (
        <div className="space-y-6 p-4">
            {/* summary cards */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <Card>
                    <CardHeader><CardTitle>Total Users</CardTitle></CardHeader>
                    <CardContent>{stats.total_users}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Total Tasks</CardTitle></CardHeader>
                    <CardContent>{stats.total_tasks}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Total Documents</CardTitle></CardHeader>
                    <CardContent>{stats.total_documents}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Total Conversations</CardTitle></CardHeader>
                    <CardContent>{stats.total_conversations}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Total Messages</CardTitle></CardHeader>
                    <CardContent>{stats.total_messages}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Avg Msgs/Conversation</CardTitle></CardHeader>
                    <CardContent>{stats.avg_messages_per_conversation.toFixed(2)}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Pipeline Runs</CardTitle></CardHeader>
                    <CardContent>{stats.total_pipeline_runs}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Avg Pipeline (ms)</CardTitle></CardHeader>
                    <CardContent>{stats.avg_pipeline_duration_ms.toFixed(0)}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Avg Tasks/User</CardTitle></CardHeader>
                    <CardContent>{stats.avg_tasks_per_user.toFixed(2)}</CardContent>
                </Card>
                <Card>
                    <CardHeader><CardTitle>Avg Docs/User</CardTitle></CardHeader>
                    <CardContent>{stats.avg_docs_per_user.toFixed(2)}</CardContent>
                </Card>
            </div>

            {/* messages over time */}
            <Card>
                <CardHeader><CardTitle>Messages Last 7 Days</CardTitle></CardHeader>
                <CardContent className="h-64">
                    <ResponsiveContainer width="100%" height="100%">
                        <LineChart data={stats.messages_last_7_days}>
                            <CartesianGrid strokeDasharray="3 3" />
                            <XAxis dataKey="day" />
                            <YAxis />
                            <Tooltip />
                            <Line type="monotone" dataKey="count" stroke="var(--primary-foreground)" />
                        </LineChart>
                    </ResponsiveContainer>
                </CardContent>
            </Card>
        </div>
    )
}
