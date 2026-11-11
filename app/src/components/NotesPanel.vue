<script setup>
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps({ item: { type: Object, default: null } })
marked.setOptions({ breaks: true, gfm: true })

const html = computed(() => (props.item?.notes ? marked.parse(props.item.notes) : ''))
</script>

<template>
  <div class="panel notes">
    <div class="panel-h">
      <span class="panel-t">Notes</span>
      <span class="panel-meta">agent record · markdown</span>
    </div>
    <div class="detail-body">
      <div v-if="html" class="md" v-html="html"></div>
      <div v-else class="detail-empty">(no notes)</div>
    </div>
  </div>
</template>
