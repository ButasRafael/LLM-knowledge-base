// src/components/DataTable.tsx
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"
import type { ReactNode } from "react"

export interface Column<T extends object> {
    key: keyof T
    label: string
    render?: (value: T[keyof T], row: T) => ReactNode
    className?: string
}

interface DataTableProps<T extends object> {
    data: T[]
    columns: Column<T>[]
}

export default function DataTable<T extends object>({
                                                        data,
                                                        columns,
                                                    }: DataTableProps<T>) {
    const defaultRender = (v: unknown): ReactNode => {
        if (v === null || v === undefined) return "–"
        if (typeof v === "boolean") return v.toString()
        if (typeof v === "object") return JSON.stringify(v) // <- stringify objects
        return v as ReactNode                                // number | string
    }

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    {columns.map((c) => (
                        <TableHead key={String(c.key)}>{c.label}</TableHead>
                    ))}
                </TableRow>
            </TableHeader>

            <TableBody>
                {data.map((row, idx) => (
                    <TableRow key={idx}>
                        {columns.map((c) => (
                            <TableCell key={String(c.key)} className={c.className}>
                                {c.render ? c.render(row[c.key], row) : defaultRender(row[c.key])}
                            </TableCell>
                        ))}
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    )
}
