<script>
  let {
    visible = false,
    title = 'Ready',
    stepDesc = '',
    percent = 0,
    details = 'Standby',
    onCancel = () => {}
  } = $props();
</script>

<div class="bottom-left-cluster">
  <div class="install-progress-card" class:active={visible} id="install-progress-card">
    <div class="progress-header">
      <div class="progress-info">
        <span class="progress-title" id="progress-title">{title}</span>
        {#if stepDesc}
          <span class="progress-step-desc" id="progress-step-desc">{stepDesc}</span>
        {/if}
      </div>
      <span class="progress-percent" id="progress-percent">{percent}%</span>
    </div>

    <div class="progress-bar-bg">
      <div class="progress-bar-fill" id="progress-bar-fill" style="width: {percent}%"></div>
    </div>

    <div class="progress-footer">
      <span class="progress-status-text" id="progress-details">{details}</span>
      <div class="progress-actions">
        <button class="text-btn text-btn-danger" id="cancel-operation-btn" onclick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>
