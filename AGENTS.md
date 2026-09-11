This is a Tauri v2 project that dispatch user input to corresponding tools and feedback the result.

# Technical stack

- Base: Tauri V2
- Language:
  - Base framework: rust
  - UI : vue, typescript, css, html, tailwind
- Package manager:
  - Frontend: bun
  - Native: cargo

## UI Framework

Use `shadcn-vue` as ui framework, it's configuration file is 'components.json'.
When need create a new component, first check/search available components from shadcn-vue, then use or modify it, use `shadcn-vue` skill if need.

# Basic layout

- src: frontend UI and app logic
- src-tauri: native framework, functions

# Useful Commands

- `bun run check:f`: compile frontend, to see whether there are errors
- `bun run check:m`: compile base, to see whether there are errors
- `bun run shadcn`: use shadcn vue cli to add/search component, view docs