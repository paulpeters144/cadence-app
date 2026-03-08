# Cadence Frontend

A modern, responsive web application for managing your personal agile workflow. Built as a PWA for a seamless experience across devices.

## Tech Stack

- **Framework**: [React](https://react.dev/) (TypeScript)
- **Build Tool**: [Vite](https://vitejs.dev/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) + [shadcn/ui](https://ui.shadcn.com/)
- **State & Data**: [React Router v7](https://reactrouter.com/) + [TanStack Query](https://tanstack.com/query)
- **Icons**: [Lucide React](https://lucide.dev/)
- **Drag & Drop**: [dnd kit](https://dndkit.com/)

## Features

- **PWA Support**: Installable on mobile and desktop with offline capabilities.
- **Agile Workflow**: Points-based task management and sprint collections.
- **Performance Tracking**: Visual dashboards for velocity and history.
- **Journaling**: Capture notes and wins for every sprint.

## Getting Started

### Environment Variables

Create a `.env` file:

```env
VITE_API_URL=http://localhost:3001
```

### Development

```powershell
pnpm run dev
```

### Building for Production

```powershell
pnpm run build
```

## Type Safety

This project uses `openapi-typescript` to generate types from the backend's OpenAPI spec. To update types:

1. Ensure the backend is running or export the `openapi.json`.
2. Run:
   ```powershell
   pnpm run generate-types
   ```
