<script setup>
import BaseModal from './BaseModal.vue'

defineProps({
  themes: { type: Array, default: () => [] },
  currentTheme: { type: String, default: 'checklist-default' },
  chime: { type: Boolean, default: true },
  tick: { type: Boolean, default: true },
  dir: { type: String, default: '' },
})
const emit = defineEmits(['preview', 'commit', 'chime', 'tick', 'close'])
</script>

<template>
  <BaseModal :width="540" @close="emit('close')">
    <div class="modal-title">Config</div>

    <div class="cfg-section">Theme</div>
    <div class="theme-grid">
      <button
        v-for="t in themes"
        :key="t.id"
        class="theme-chip"
        :class="{ active: t.id === currentTheme }"
        @mouseenter="emit('preview', t.id)"
        @mouseleave="emit('preview', currentTheme)"
        @click="emit('commit', t.id)"
      >{{ t.name }}</button>
    </div>

    <div class="cfg-row">
      <span class="cfg-label">Completion chime</span>
      <button class="toggle" :class="{ on: chime }" @click="emit('chime', !chime)">
        <span class="knob"></span><span class="tg-txt">{{ chime ? 'On' : 'Off' }}</span>
      </button>
    </div>

    <div class="cfg-row">
      <span class="cfg-label">Tick sound <span class="muted">· on agent ticks</span></span>
      <button class="toggle" :class="{ on: tick }" @click="emit('tick', !tick)">
        <span class="knob"></span><span class="tg-txt">{{ tick ? 'On' : 'Off' }}</span>
      </button>
    </div>

    <div class="cfg-row">
      <span class="cfg-label">Store folder</span>
      <span class="cfg-val mono">{{ dir }}</span>
    </div>

    <div class="modal-hint">hover a theme to preview · click to apply · <kbd>Esc</kbd> close</div>
  </BaseModal>
</template>
