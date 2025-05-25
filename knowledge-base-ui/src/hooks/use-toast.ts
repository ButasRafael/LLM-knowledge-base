/* ──────────────────────────────────────────────────────────
   use-toast.tsx  –  lightweight toast manager (client side)
   Inspired by react-hot-toast ──────────────────────────── */

"use client"

import * as React from "react"
import type { ToastActionElement, ToastProps } from "@/components/ui/toast"

/* ------------------------------------------------------------------ */
/* Constants                                                           */
/* ------------------------------------------------------------------ */

const TOAST_LIMIT        = 1            // max parallel toasts
const TOAST_REMOVE_DELAY = 1_000_000    // ms before GC

/* Centralised action “enum” – referenced at runtime + compile-time */
export const ACTION = {
  ADD     : "ADD_TOAST",
  UPDATE  : "UPDATE_TOAST",
  DISMISS : "DISMISS_TOAST",
  REMOVE  : "REMOVE_TOAST",
} as const

/* ------------------------------------------------------------------ */
/* Types                                                               */
/* ------------------------------------------------------------------ */

type ToasterToast = ToastProps & {
  id          : string
  title?      : React.ReactNode
  description?: React.ReactNode
  action?     : ToastActionElement
}

type Action =
    | { type: typeof ACTION.ADD;     toast  : ToasterToast           }
    | { type: typeof ACTION.UPDATE;  toast  : Partial<ToasterToast>  }
    | { type: typeof ACTION.DISMISS; toastId?: ToasterToast["id"]    }
    | { type: typeof ACTION.REMOVE;  toastId?: ToasterToast["id"]    }

interface State {
  toasts: ToasterToast[]
}

/* ------------------------------------------------------------------ */
/* Helpers                                                             */
/* ------------------------------------------------------------------ */

let counter = 0
const nextId = () => ((counter = (counter + 1) % Number.MAX_SAFE_INTEGER) , `${counter}`)

const timeouts = new Map<string, ReturnType<typeof setTimeout>>()

/* Remove-later queue */
function queueRemoval(id: string) {
  if (timeouts.has(id)) return
  const t = setTimeout(() => {
    timeouts.delete(id)
    dispatch({ type: ACTION.REMOVE, toastId: id })
  }, TOAST_REMOVE_DELAY)
  timeouts.set(id, t)
}

/* ------------------------------------------------------------------ */
/* Reducer                                                             */
/* ------------------------------------------------------------------ */

export const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case ACTION.ADD:
      return { ...state, toasts: [action.toast, ...state.toasts].slice(0, TOAST_LIMIT) }

    case ACTION.UPDATE:
      return {
        ...state,
        toasts: state.toasts.map(t => t.id === action.toast.id ? { ...t, ...action.toast } : t),
      }

    case ACTION.DISMISS: {
      const { toastId } = action

      if (toastId) {
        queueRemoval(toastId)
      } else {
        // no ID ⇒ dismiss **all** active toasts
        state.toasts.forEach(t => queueRemoval(t.id))
      }

      return {
        ...state,
        toasts: state.toasts.map(t =>
            !toastId || t.id === toastId ? { ...t, open: false } : t
        ),
      }
    }


    case ACTION.REMOVE:
      return {
        ...state,
        toasts: action.toastId
            ? state.toasts.filter(t => t.id !== action.toastId)
            : [],
      }
  }
}

/* ------------------------------------------------------------------ */
/* Global store (very tiny)                                            */
/* ------------------------------------------------------------------ */

let memState: State = { toasts: [] }
const listeners: Array<(s: State) => void> = []

function dispatch(action: Action) {
  memState = reducer(memState, action)
  listeners.forEach(fn => fn(memState))
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

type ToastInput = Omit<ToasterToast, "id">

export function toast(props: ToastInput) {
  const id = nextId()

  const update  = (p: Partial<ToasterToast>) => dispatch({ type: ACTION.UPDATE,  toast: { ...p, id } })
  const dismiss = ()                        => dispatch({ type: ACTION.DISMISS, toastId: id })

  dispatch({
    type : ACTION.ADD,
    toast: {
      ...props,
      id,
      open        : true,
      onOpenChange: open => { if (!open) dismiss() },
    },
  })

  return { id, update, dismiss }
}

export function useToast() {
  const [state, setState] = React.useState<State>(memState)

  React.useEffect(() => {
    listeners.push(setState)
    return () => {
      const i = listeners.indexOf(setState)
      if (i !== -1) listeners.splice(i, 1)
    }
  }, [])

  return {
    ...state,
    toast,
    dismiss: (id?: string) => dispatch({ type: ACTION.DISMISS, toastId: id }),
  }
}
