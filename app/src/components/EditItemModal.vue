<script setup>
import { ref, onMounted } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({ label: { type: String, default: '' }, detail: { type: String, default: '' } })
const emit = defineEmits(['save', 'close'])

const label = ref(props.label)
const detail = ref(props.detail)
const titleEl = ref(null)
onMounted(() => titleEl.value?.focus())

function save() { emit('save', { label: label.value, detail: detail.value }) }
</script>

<template>
  <BaseModal :width="640" @close="emit('close')">
    <div class="modal-title">Edit item</div>
    <label class="modal-label">Title</label>
    <input ref="titleEl" v-model="label" class="modal-input" @keydown.enter="save" />
    <label class="modal-label">Description <span class="muted">· markdown</span></label>
    <textarea v-model="detail" class="modal-textarea" rows="7" @keydown.ctrl.enter="save"></textarea>
    <div class="modal-actions">
      <button class="btn" @click="emit('close')">Cancel</button>
      <button class="btn primary" @click="save">Save</button>
    </div>
    <div class="modal-hint"><kbd>Ctrl</kbd>+<kbd>Enter</kbd> save · <kbd>Esc</kbd> cancel</div>
  </BaseModal>
</template>
