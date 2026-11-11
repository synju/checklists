// In-memory sample store for browser preview (no Electron / Rust service).
// Generic demo content so the UI has something realistic to render.

const data = {
  'release-v1': {
    id: 'release-v1',
    name: 'Ship v1.0',
    items: [
      { label: '1. Draft the release checklist', done: true },
      { label: '2. Freeze the feature set', done: true },
      { label: '3. Write migration notes', done: true },
      { label: '4. Bump version numbers', done: true },
      { label: '5. Update the changelog', done: true },
      { label: '6. Run the full test suite', done: true },
      { label: '7. Build release artifacts', done: true },
      { label: '8. Smoke-test the installer', done: true },
      { label: '9. Tag the release', done: true },
      {
        label: '10. Publish the release notes',
        done: false,
        detail: '## Publish the release notes\n\nWrite the user-facing notes, highlight **breaking changes**, and link the full `CHANGELOG`.\n\n- Summarize new features\n- Call out breaking changes\n- Thank contributors\n\nSee the [release guide](https://example.com/release-guide).',
        notes: '**Context:** last release the notes shipped a day late — draft them during the freeze next time.\n\n```sql\nselect tag, published_at from releases order by published_at desc limit 5;\n```\n\nTemplate: [release-notes.md](https://example.com/templates).',
      },
      { label: '11. Announce on the blog', done: false },
      { label: '12. Update the docs site', done: false },
      { label: '13. Notify beta testers', done: false },
      { label: '14. Monitor error rates post-launch', done: false },
    ],
  },
  'website': {
    id: 'website',
    name: 'Website redesign',
    items: [
      { label: '1. Audit the current site', done: true },
      { label: '2. Wireframe the new layout', done: true },
      { label: '3. Design the home page', done: true },
      {
        label: '4. Build the component library',
        done: true,
        detail: 'Build reusable `Button`, `Card`, and `Nav` components with tokens for color and spacing.',
        notes: '**Decision:** went with CSS variables for theming.\n\n```js\nexport const tokens = { accent: \'var(--accent)\' }\n```\n\nRef: [design system](https://example.com/ds).',
      },
      { label: '5. Migrate the content', done: true },
      { label: '6. Launch', done: true },
    ],
  },
  'api-cleanup': {
    id: 'api-cleanup',
    name: 'API cleanup',
    items: [
      { label: '1. List deprecated endpoints', done: true },
      { label: '2. Add deprecation warnings', done: true },
      { label: '3. Write the migration guide', done: true },
      { label: '4. Remove dead routes', done: false },
      { label: '5. Consolidate response shapes', done: false },
      { label: '6. Add request validation', done: false },
      { label: '7. Update the OpenAPI spec', done: false },
      { label: '8. Verify with the client team', done: false },
    ],
  },
  'onboarding-docs': {
    id: 'onboarding-docs',
    name: 'Onboarding docs',
    items: [
      { label: '1. Outline the getting-started guide', done: false },
      { label: '2. Write the install steps', done: false },
      { label: '3. Add a quickstart example', done: false },
      { label: '4. Document the config options', done: false },
      { label: '5. Review with a new hire', done: false },
    ],
  },
}

function summary(cl) {
  const total = cl.items.length
  const done = cl.items.filter((i) => i.done).length
  return {
    id: cl.id,
    name: cl.name,
    displayName: cl.name || cl.id,
    done,
    total,
    percent: total ? Math.round((done / total) * 100) : 0,
    complete: total > 0 && done === total,
  }
}

export function mockList() {
  return { dir: '(browser preview — sample data)', checklists: Object.values(data).map(summary) }
}

export function mockGet(id) {
  const cl = data[id]
  if (!cl) throw new Error('not found: ' + id)
  return {
    ...summary(cl),
    items: cl.items.map((i) => ({
      label: i.label,
      done: i.done,
      detail: i.detail || '',
      ...(i.notes ? { notes: i.notes } : {}),
    })),
  }
}

export function mockToggle(id, index, done) {
  const cl = data[id]
  if (cl && cl.items[index]) {
    cl.items[index].done = done === undefined ? !cl.items[index].done : done
    return { id, index, done: cl.items[index].done }
  }
  return { id, index, done: false }
}

export function mockRename(id, name) {
  const cl = data[id]
  if (!cl) return { id, name: null }
  const n = name && name.trim() && name.trim() !== id ? name.trim() : null
  if (n) cl.name = n
  else delete cl.name
  return { id, name: n }
}

export function mockEditItem(id, index, label, detail) {
  const cl = data[id]
  if (cl && cl.items[index]) {
    cl.items[index].label = label
    cl.items[index].detail = detail
  }
  return mockGet(id)
}

export function mockAddItem(id, label) {
  const cl = data[id]
  if (cl) cl.items.push({ label, done: false })
  return mockGet(id)
}

export function mockDeleteItem(id, index) {
  const cl = data[id]
  if (cl && cl.items[index] !== undefined) cl.items.splice(index, 1)
  return mockGet(id)
}

export function mockDelete(id) {
  delete data[id]
  return { id, deleted: true }
}

export function mockCreate(id, name) {
  if (!data[id]) data[id] = { id, name: name || undefined, items: [] }
  return summary(data[id])
}

let cfg = { dir: '(browser preview — sample data)', theme: 'checklist-default', chime: true, tick: true }
export function mockConfig() {
  return { ...cfg }
}
export function mockSetConfig(patch) {
  cfg = { ...cfg, ...patch }
  return { theme: cfg.theme, chime: cfg.chime, tick: cfg.tick }
}
