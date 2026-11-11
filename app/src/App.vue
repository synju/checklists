<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import * as api from './api.js'
import { playChime } from './chime.js'
import Sidebar from './components/Sidebar.vue'
import ProgressHeader from './components/ProgressHeader.vue'
import Checklist from './components/Checklist.vue'
import DetailPanel from './components/DetailPanel.vue'
import NotesPanel from './components/NotesPanel.vue'
import DetailView from './components/DetailView.vue'
import ActionsMenu from './components/ActionsMenu.vue'
import EditItemModal from './components/EditItemModal.vue'
import PromptModal from './components/PromptModal.vue'
import ConfirmModal from './components/ConfirmModal.vue'
import ConfigModal from './components/ConfigModal.vue'
import { THEMES, applyTheme } from './themes.js'

const checklists = ref([])
const activeId = ref(null)
const active = ref(null)
const cursor = ref(0)
const mode = ref('list')
const storeDir = ref('')
const toast = ref(false)
const modal = ref(null)
const theme = ref('checklist-default')
const chimeEnabled = ref(true)
const tickEnabled = ref(true)
const sidebarVisible = ref(loadPref('sidebar', true))
const focusMode = ref(false)
let wasComplete = false
let prevDone = 0
let unsubscribe = () => {}

function loadPref(key, def) {
  try { const v = localStorage.getItem('cl.' + key); return v === null ? def : JSON.parse(v) } catch { return def }
}
function savePref(key, val) {
  try { localStorage.setItem('cl.' + key, JSON.stringify(val)) } catch { /* ignore */ }
}
function toggleSidebar() { sidebarVisible.value = !sidebarVisible.value; focusMode.value = false; savePref('sidebar', sidebarVisible.value) }
function toggleFocus() { focusMode.value = !focusMode.value }

const cursorItem = computed(() => active.value?.items?.[cursor.value] || null)

// --- data ---
async function loadList() {
  const res = await api.call('list')
  checklists.value = res.checklists || []
  storeDir.value = res.dir || ''
  if (!activeId.value && checklists.value.length) await selectChecklist(checklists.value[0].id)
}
async function loadActive() {
  if (!activeId.value) { active.value = null; return }
  active.value = await api.call('get', { id: activeId.value })
  const n = active.value.items?.length || 0
  if (cursor.value >= n) cursor.value = Math.max(0, n - 1)
}
async function refreshActiveList() { await Promise.all([loadActive(), loadList()]) }

async function selectChecklist(id) {
  activeId.value = id
  cursor.value = 0
  mode.value = 'list'
  await loadActive()
  wasComplete = active.value?.complete || false
  prevDone = active.value?.done || 0
}
function switchChecklist(delta) {
  if (!checklists.value.length) return
  const ids = checklists.value.map((c) => c.id)
  let i = ids.indexOf(activeId.value)
  if (i < 0) i = 0
  selectChecklist(ids[(i + delta + ids.length) % ids.length])
}
function moveCursor(delta) {
  const n = active.value?.items?.length || 0
  if (!n) return
  cursor.value = Math.max(0, Math.min(n - 1, cursor.value + delta))
  nextTick(() => document.querySelector('.row.active')?.scrollIntoView({ block: 'nearest' }))
}

// --- chime ---
async function toggle(index) {
  const beforeComplete = active.value?.complete || false
  const beforeDone = prevDone
  await api.call('toggle', { id: activeId.value, index })
  await refreshActiveList()
  settle(beforeComplete, beforeDone, false) // manual toggle: no tick sound
}
// Fire the completion chime (and toast) or, for an agent tick, the tick sound.
function settle(beforeComplete, beforeDone, allowTick) {
  const nowComplete = active.value?.complete || false
  const nowDone = active.value?.done || 0
  if (nowComplete && !beforeComplete) celebrate()
  else if (allowTick && nowDone > beforeDone && tickEnabled.value) playTick()
  wasComplete = nowComplete
  prevDone = nowDone
}
function celebrate() {
  if (chimeEnabled.value) playChime()
  toast.value = true
  setTimeout(() => (toast.value = false), 3200)
}

// --- config / themes ---
async function loadConfig() {
  const cfg = await api.call('get_config')
  theme.value = cfg.theme || 'checklist-default'
  chimeEnabled.value = cfg.chime !== false
  tickEnabled.value = cfg.tick !== false
  applyTheme(theme.value)
}
function openConfig() { modal.value = { kind: 'config' } }
function onPreviewTheme(id) { applyTheme(id) }
function onCommitTheme(id) { theme.value = id; applyTheme(id); api.call('set_config', { theme: id }) }
function onToggleChime(v) { chimeEnabled.value = v; api.call('set_config', { chime: v }) }
function onToggleTick(v) { tickEnabled.value = v; api.call('set_config', { tick: v }) }
function onCloseConfig() { applyTheme(theme.value); closeModal() }
async function refreshAll() {
  const beforeComplete = wasComplete
  const beforeDone = prevDone
  await loadList(); await loadActive()
  settle(beforeComplete, beforeDone, true) // a changed event may be an agent tick
}

function openDetail() { if (active.value?.items?.length) mode.value = 'detail' }
function closeDetail() { mode.value = 'list' }

// --- modals ---
function closeModal() { modal.value = null }

function itemContext(index) {
  cursor.value = index
  modal.value = { kind: 'actions', ctx: 'item', title: 'Item', index, options: [
    { id: 'edit', label: 'Edit Item' },
    { id: 'delete', label: 'Delete Item', danger: true },
  ] }
}
function checklistContext(id) {
  modal.value = { kind: 'actions', ctx: 'checklist', title: 'Checklist', id, options: [
    { id: 'rename', label: 'Rename Checklist' },
    { id: 'delete', label: 'Delete Checklist', danger: true },
  ] }
}
function onAction(actionId) {
  const m = modal.value
  if (!m) return
  if (m.ctx === 'item') {
    if (actionId === 'edit') openEdit(m.index)
    else if (actionId === 'delete') confirmDeleteItem(m.index)
  } else {
    if (actionId === 'rename') openRename(m.id)
    else if (actionId === 'delete') confirmDeleteChecklist(m.id)
  }
}

function openEdit(index) {
  const it = active.value?.items?.[index]
  if (!it) return
  modal.value = { kind: 'edit', index, label: it.label, detail: it.detail || '' }
}
async function saveEdit({ label, detail }) {
  const index = modal.value.index
  closeModal()
  await api.call('edit_item', { id: activeId.value, index, label, detail })
  await refreshActiveList()
}

function openRename(id) {
  const cl = checklists.value.find((c) => c.id === id)
  modal.value = { kind: 'prompt', title: 'Rename checklist', label: 'Display name', value: cl?.name || '', placeholder: id, okText: 'Save', onSubmit: (v) => doRename(id, v) }
}
async function doRename(id, name) { closeModal(); await api.call('rename', { id, name }); await refreshActiveList() }

function confirmDeleteItem(index) {
  const it = active.value?.items?.[index]
  modal.value = { kind: 'confirm', message: `Delete this item?\n\n"${it?.label || ''}"`, okText: 'Delete', onYes: () => doDeleteItem(index) }
}
async function doDeleteItem(index) { closeModal(); await api.call('delete_item', { id: activeId.value, index }); await refreshActiveList() }

function confirmDeleteChecklist(id) {
  const cl = checklists.value.find((c) => c.id === id)
  modal.value = { kind: 'confirm', message: `Delete "${cl?.displayName || id}"?\nThis removes ${id}.json.`, okText: 'Delete', onYes: () => doDeleteChecklist(id) }
}
async function doDeleteChecklist(id) {
  closeModal()
  await api.call('delete', { id })
  if (activeId.value === id) activeId.value = null
  await loadList()
  if (!activeId.value && checklists.value.length) await selectChecklist(checklists.value[0].id)
  else await loadActive()
}

function openNewChecklist() {
  modal.value = { kind: 'prompt', title: 'New checklist', label: 'File id (becomes <id>.json)', value: '', placeholder: 'e.g. my-task', okText: 'Create', onSubmit: (v) => doCreate(v) }
}
async function doCreate(id) {
  id = (id || '').trim()
  closeModal()
  if (!id) return
  try { await api.call('create', { id }) } catch (e) { /* duplicate/invalid: ignore */ }
  await loadList()
  await selectChecklist(id)
}

function openAddItem() {
  if (!activeId.value) return
  modal.value = { kind: 'prompt', title: 'Add item', label: 'Label', value: '', placeholder: 'New task…', okText: 'Add', onSubmit: (v) => doAddItem(v) }
}
async function doAddItem(label) {
  label = (label || '').trim()
  closeModal()
  if (!label) return
  await api.call('add_item', { id: activeId.value, label })
  await refreshActiveList()
}

// --- keyboard ---
function onKey(e) {
  if (modal.value) return // modals handle their own keys
  const t = e.target
  if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
  if ((e.ctrlKey || e.metaKey) && (e.key === 'l' || e.key === 'L')) { toggleSidebar(); e.preventDefault(); return }
  if (mode.value === 'list') {
    switch (e.key) {
      case 'ArrowDown': case 'j': moveCursor(1); e.preventDefault(); break
      case 'ArrowUp': case 'k': moveCursor(-1); e.preventDefault(); break
      case ' ': toggle(cursor.value); e.preventDefault(); break
      case 'Enter': openDetail(); e.preventDefault(); break
      case 'ArrowLeft': switchChecklist(-1); e.preventDefault(); break
      case 'ArrowRight': switchChecklist(1); e.preventDefault(); break
      case 'e': if (active.value?.items?.length) { openEdit(cursor.value); e.preventDefault() } break
      case 'Delete': if (active.value?.items?.length) { confirmDeleteItem(cursor.value); e.preventDefault() } break
      case 'a': openAddItem(); e.preventDefault(); break
      case 'r': if (activeId.value) { openRename(activeId.value); e.preventDefault() } break
      case 'n': openNewChecklist(); e.preventDefault(); break
      case 'c': openConfig(); e.preventDefault(); break
      case 'f': toggleFocus(); e.preventDefault(); break
    }
  } else if (e.key === 'Escape') {
    closeDetail(); e.preventDefault()
  }
}

onMounted(async () => {
  await loadConfig()
  await loadList()
  unsubscribe = api.onEvent(() => refreshAll())
  window.addEventListener('keydown', onKey)
})
onUnmounted(() => { unsubscribe(); window.removeEventListener('keydown', onKey) })
</script>

<template>
  <div class="app-window">
    <!-- title bar -->
    <div class="titlebar">
      <div class="brand">
        <div class="logo">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
            <path d="M5 12.5l4.5 4.5L19 7.5" stroke="var(--bg)" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>
        <span class="brand-name">Checklist</span>
        <span class="chip"><span class="dot"></span>{{ api.isLive ? 'live' : 'preview' }}</span>
      </div>
      <div class="titlebar-actions">
        <button class="tbtn" title="Toggle sidebar (Ctrl+L)" @click="toggleSidebar">☰</button>
        <button class="tbtn" @click="openNewChecklist">＋ Checklist</button>
        <button class="tbtn" :disabled="!activeId" @click="openAddItem">＋ Item</button>
        <button class="tbtn" title="Config" @click="openConfig">⚙ Config</button>
      </div>
      <div class="winbtns">
        <button class="winbtn" aria-label="Minimize" @click="api.win.minimize()">
          <svg width="12" height="12" viewBox="0 0 13 13"><line x1="2.5" y1="9.5" x2="10.5" y2="9.5" stroke="currentColor" stroke-width="1.3" /></svg>
        </button>
        <button class="winbtn" aria-label="Maximize" @click="api.win.toggleMaximize()">
          <svg width="11" height="11" viewBox="0 0 13 13"><rect x="2.5" y="2.5" width="8" height="8" rx="1.3" stroke="currentColor" stroke-width="1.3" fill="none" /></svg>
        </button>
        <button class="winbtn close" aria-label="Close" @click="api.win.close()">
          <svg width="11" height="11" viewBox="0 0 13 13"><line x1="3" y1="3" x2="10" y2="10" stroke="currentColor" stroke-width="1.3" /><line x1="10" y1="3" x2="3" y2="10" stroke="currentColor" stroke-width="1.3" /></svg>
        </button>
      </div>
    </div>

    <!-- completion toast -->
    <transition name="toast">
      <div v-if="toast" class="toast">
        <div class="ic">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none"><path d="M5 12.5l4.5 4.5L19 7.5" stroke="var(--green)" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </div>
        <div>
          <div class="t1">Checklist complete</div>
          <div class="t2">stage clear — chime played</div>
        </div>
      </div>
    </transition>

    <!-- content -->
    <div class="content">
      <Sidebar v-if="sidebarVisible && !focusMode" :checklists="checklists" :active-id="activeId" @select="selectChecklist" @context="checklistContext" />
      <div class="maincol">
        <ProgressHeader :cl="active" />
        <template v-if="mode === 'list'">
          <Checklist :items="active?.items || []" :cursor="cursor" @toggle="toggle" @select="(i) => (cursor = i)" @open="openDetail" @context="itemContext" />
          <DetailPanel v-if="!focusMode" :item="cursorItem" />
          <NotesPanel v-if="!focusMode && cursorItem?.notes" :item="cursorItem" />
        </template>
        <DetailView v-else :item="cursorItem" :name="active?.displayName" @close="closeDetail" />
      </div>
    </div>

    <!-- footer -->
    <div class="footer">
      <div class="hints">
        <span><kbd>↑</kbd><kbd>↓</kbd> move</span><span class="sep">·</span>
        <span><kbd>space</kbd> toggle</span><span class="sep">·</span>
        <span><kbd>enter</kbd> detail</span><span class="sep">·</span>
        <span><kbd>e</kbd> edit</span><span class="sep">·</span>
        <span><kbd>a</kbd> add</span><span class="sep">·</span>
        <span><kbd>f</kbd> focus</span><span class="sep">·</span>
        <span>right-click for actions</span>
      </div>
      <div class="hints">
        <span class="dotlive"></span>
        <span>{{ api.isLive ? 'live · watching' : 'preview · sample data' }}</span>
        <span class="sep">·</span>
        <span class="mono" style="color:#787878">{{ storeDir }}</span>
      </div>
    </div>

    <!-- modals -->
    <ActionsMenu v-if="modal?.kind === 'actions'" :title="modal.title" :options="modal.options" @pick="onAction" @close="closeModal" />
    <EditItemModal v-else-if="modal?.kind === 'edit'" :label="modal.label" :detail="modal.detail" @save="saveEdit" @close="closeModal" />
    <PromptModal v-else-if="modal?.kind === 'prompt'" :title="modal.title" :label="modal.label" :value="modal.value" :placeholder="modal.placeholder" :ok-text="modal.okText" @submit="modal.onSubmit" @close="closeModal" />
    <ConfirmModal v-else-if="modal?.kind === 'confirm'" :message="modal.message" :ok-text="modal.okText" @yes="modal.onYes" @close="closeModal" />
    <ConfigModal v-else-if="modal?.kind === 'config'" :themes="THEMES" :current-theme="theme" :chime="chimeEnabled" :tick="tickEnabled" :dir="storeDir" @preview="onPreviewTheme" @commit="onCommitTheme" @chime="onToggleChime" @tick="onToggleTick" @close="onCloseConfig" />
  </div>
</template>
