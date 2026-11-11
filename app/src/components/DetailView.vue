<script setup>
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps({ item: { type: Object, default: null }, name: { type: String, default: '' } })
defineEmits(['close'])
marked.setOptions({ breaks: true, gfm: true })

const detailHtml = computed(() => (props.item?.detail ? marked.parse(props.item.detail) : ''))
const notesHtml = computed(() => (props.item?.notes ? marked.parse(props.item.notes) : ''))
</script>

<template>
  <div class="panel checklist focused">
    <div class="panel-h">
      <span class="panel-t">{{ name }} · detail</span>
      <span class="panel-meta"><kbd>Esc</kbd> back · <kbd>j</kbd><kbd>k</kbd> scroll</span>
    </div>
    <div class="detail-body" style="padding:20px 26px">
      <template v-if="item">
        <div style="font-size:15px;font-weight:700;color:#fff;margin-bottom:16px">{{ item.label }}</div>
        <div v-if="detailHtml" class="md" v-html="detailHtml"></div>
        <div v-else class="detail-empty">(no detail)</div>
        <template v-if="notesHtml">
          <div class="notes-h">Notes</div>
          <div class="md" v-html="notesHtml"></div>
        </template>
      </template>
      <div v-else class="detail-empty">No item selected</div>
    </div>
  </div>
</template>
