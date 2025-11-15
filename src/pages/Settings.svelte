<script lang="ts">
  import { router } from '../states';
  
  let theme = 'light';
  let language = 'zh';
  
  function saveSettings() {
    // 保存设置
    alert('设置已保存');
    router.pop('slide-right');
  }
</script>

<div class="settings-page">
  <h1>设置</h1>
  <div>
    <label>主题:</label>
    <select bind:value={theme}>
      <option value="light">浅色</option>
      <option value="dark">深色</option>
    </select>
  </div>
  <div>
    <label>语言:</label>
    <select bind:value={language}>
      <option value="zh">中文</option>
      <option value="en">English</option>
    </select>
  </div>
  <button onclick={saveSettings}>保存</button>
  <button onclick={() => router.pop('slide-right')}>返回</button>
</div>

<style>
  .settings-page {
    padding: 20px;
  }
  
  div {
    margin-bottom: 15px;
  }
  
  label {
    display: block;
    margin-bottom: 5px;
  }
  
  select {
    width: 100%;
    padding: 8px;
    box-sizing: border-box;
  }
  
  button {
    margin-right: 10px;
    margin-top: 10px;
  }
</style>