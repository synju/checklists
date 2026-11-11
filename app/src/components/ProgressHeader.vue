<script setup>
defineProps({ cl: { type: Object, default: null } })
</script>

<template>
  <div class="panel" v-if="cl">
    <div class="panel-h">
      <span class="panel-t">Progress</span>
      <span class="panel-meta">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
          <path d="M4 9v6h4l5 4V5L8 9H4z" :fill="cl.complete ? 'var(--green)' : '#4c7a5c'" />
          <path d="M16 9a4 4 0 0 1 0 6" :stroke="cl.complete ? 'var(--green)' : '#4c7a5c'" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        {{ cl.complete ? 'chime played' : 'chime armed' }}
      </span>
    </div>
    <div class="ph-body">
      <div class="ph-row">
        <div>
          <div class="ph-name">{{ cl.displayName }}</div>
          <div class="ph-file mono">checklists/{{ cl.id }}.json</div>
        </div>
        <div>
          <div class="ph-pct">{{ cl.percent }}%</div>
          <div class="ph-count">{{ cl.done }} / {{ cl.total }} complete</div>
        </div>
      </div>
      <div class="bar-track"><div class="bar-fill" :style="{ width: cl.percent + '%' }"></div></div>
    </div>
  </div>
</template>
