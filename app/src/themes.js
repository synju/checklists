// Theme ids map to :root[data-theme="…"] blocks in style.css.
export const THEMES = [
  { id: 'checklist-default', name: 'Checklist (default)' },
  { id: 'nord', name: 'Nord' },
  { id: 'dracula', name: 'Dracula' },
  { id: 'gruvbox', name: 'Gruvbox' },
  { id: 'tokyo-night', name: 'Tokyo Night' },
  { id: 'catppuccin', name: 'Catppuccin' },
]

export function applyTheme(id) {
  const root = document.documentElement
  if (!id || id === 'checklist-default') root.removeAttribute('data-theme')
  else root.setAttribute('data-theme', id)
}
