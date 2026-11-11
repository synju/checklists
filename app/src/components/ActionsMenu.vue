<script setup>
import { ref, onMounted } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({ title: String, options: { type: Array, default: () => [] } })
const emit = defineEmits(['pick', 'close'])

const idx = ref(0)
const listEl = ref(null)
onMounted(() => listEl.value?.focus())

function onKey(e) {
  if (e.key === 'ArrowDown' || e.key === 'j') { idx.value = Math.min(props.options.length - 1, idx.value + 1); e.preventDefault() }
  else if (e.key === 'ArrowUp' || e.key === 'k') { idx.value = Math.max(0, idx.value - 1); e.preventDefault() }
  else if (e.key === 'Enter') { emit('pick', props.options[idx.value].id); e.preventDefault() }
}
</script>

<template>
  <BaseModal :width="380" @close="emit('close')">
    <div class="modal-title">{{ title }}</div>
    <div ref="listEl" class="actions-list" tabindex="0" @keydown="onKey">
      <div
        v-for="(o, i) in options"
        :key="o.id"
        class="action-row"
        :class="{ active: i === idx, danger: o.danger }"
        @mouseenter="idx = i"
        @click="emit('pick', o.id)"
      >{{ o.label }}</div>
    </div>
    <div class="modal-hint"><kbd>j</kbd><kbd>k</kbd> move · <kbd>enter</kbd> run · <kbd>esc</kbd> cancel</div>
  </BaseModal>
</template>
