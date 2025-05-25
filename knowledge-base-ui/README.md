# Knowledge Base Frontend

A rich interactive web client for the Knowledge Base application, built with Next.js, TypeScript, Tailwind CSS, and Shadcn/UI components.

---

## Table of Contents

* [Features](#features)
* [Tech Stack](#tech-stack)
* [Prerequisites](#prerequisites)
* [Getting Started](#getting-started)
* [Environment Variables](#environment-variables)
* [Project Structure](#project-structure)
* [Routing & Middleware](#routing--middleware)
* [Styling & Theming](#styling--theming)
* [Key Modules](#key-modules)

    * [API Wrapper (`src/lib/api.ts`)](#api-wrapper-srclibapits)
    * [Authentication Helper (`src/lib/auth.ts`)](#authentication-helper-srclibauthts)
    * [UI Components](#ui-components)
    * [Pages & App Directory](#pages--app-directory)
* [Building & Deployment](#building--deployment)
* [Contributing](#contributing)
* [License](#license)

---

## Features

* **Authentication**: Login, register, JWT-based cookie auth, route guards
* **Chat**: Create/list conversations, send/receive messages with live polling
* **Tasks**: CRUD interface for user tasks
* **Documents**: Upload, list, rename, delete; integrated drag‑and‑drop UI
* **Users**: Admin UI for create/edit/delete users
* **Profile**: Self‑service view & password change
* **Metrics**: Admin dashboard charts powered by Recharts
* **Responsive**: Sidebar/Topbar adapt between desktop & mobile
* **Dark Mode**: Toggle with Tailwind CSS class switching

## Tech Stack

* **Framework**: Next.js 13 (App Router)
* **Language**: TypeScript
* **Styling**: Tailwind CSS, Tailwind Animate plugin
* **UI Components**: [Shadcn](https://ui.shadcn.com/) & Lucide icons
* **State & Data Fetching**: React hooks + built‑in `fetch`
* **Charts**: Recharts
* **Markdown**: React Markdown + GFM support

## Prerequisites

* Node.js ≥ 18
* NPM or Yarn or PNPM or Bun
* Rust backend running locally (default at `http://localhost:8000`)

## Getting Started

1. **Install dependencies**

   ```bash
   npm install
   # or yarn
   # or pnpm install
   ```

2. **Set environment variables**
   Create a `.env.local` in project root:

   ```env
   NEXT_PUBLIC_API_BASE=http://localhost:8000
   ```

3. **Run development server**

   ```bash
   npm run dev
   # or yarn dev
   # or pnpm dev
   ```

   Open [http://localhost:3000](http://localhost:3000) in your browser.

4. **Build for production**

   ```bash
   npm run build
   npm start
   ```

## Environment Variables

| Variable               | Purpose                                                     |
| ---------------------- | ----------------------------------------------------------- |
| `NEXT_PUBLIC_API_BASE` | Base URL of the backend API (e.g., `http://localhost:8000`) |

## Project Structure

```
├── public/            # Static assets (icons, favicon)
├── src/
│   ├── app/           # Next.js App Router pages & layouts
│   ├── components/    # Shared UI components (Sidebar, TopBar, DataTable)
│   ├── hooks/         # Custom React hooks (e.g., use-toast)
│   ├── lib/           # Utility libraries (api, auth, utils)
│   ├── styles/        # Tailwind globals
│   └── ...
├── tailwind.config.ts
├── next.config.ts
├── middleware.ts      # Route guards & redirects
├── tsconfig.json
├── .eslintrc.js
└── README.md
```

## Routing & Middleware

* **Public routes**: `/login`, `/register`, static assets
* **Client routes**: guarded by `middleware.ts` (chat, tasks, documents, profile)
* **Admin routes**: `/admin/*` (metrics, users) with role check in middleware

Middleware lives at `middleware.ts` and uses cookie inspection to redirect unauthorized users to `/login`.

## Styling & Theming

* **Tailwind** with custom CSS variables in `globals.css`
* **Dark mode** toggled by adding/removing `class="dark"` on `<html>`
* **Animate plugin** for subtle UI transitions
* **Shadcn/UI** components in `src/components/ui` for cards, buttons, dialogs, etc.

## Key Modules

### API Wrapper (`src/lib/api.ts`)

A thin wrapper around `fetch` that:

* Prefixes URLs with `NEXT_PUBLIC_API_BASE`
* Automatically includes credentials
* Parses JSON and throws on HTTP errors

### Authentication Helper (`src/lib/auth.ts`)

* `getCtxFromCookie()` reads `auth-token` cookie server-side
* Decodes identifier + expiry
* Exposes `{ username, isAdmin, exp }` for layout

### UI Components

* **Sidebar**: Desktop/mobile navigation with auto‑detect admin items
* **TopBar**: Theme toggle, logout, avatar
* **DataTable**: Generic table rendering any array of objects
* **Card / Dialog / Sheet**: Shadcn for structured layouts & forms

### Pages & App Directory

* **Layouts**: `RootLayout` applies auth guard & chrome
* **Dashboard**: Quick links & recent uploads
* **Chat**: Two‑panel conversation view with polling
* **Tasks / Documents / Users**: Admin CRUD interfaces
* **Profile**: Self‑service user details & password change
* **Metrics**: Recharts line chart and summary cards
* **Auth**: Register & Login flows

## Building & Deployment

1. **Build**

   ```bash
   npm run build
   ```

2. **Preview**

   ```bash
   npm run start
   ```

3. **Docker**
   You can containerize with a simple `Dockerfile` or deploy on Vercel.


## License

[MIT](LICENSE)
