<script lang="ts">
  import { router } from '../lib/runes';
  
  let username = '';
  let password = '';
  
  function handleLogin() {
    // 模拟登录
    if (username && password) {
      localStorage.setItem('authToken', 'fake-token');
      router.push('settings', { user: username }, 'slide-left');
    }
  }
</script>

<div class="sign-page">
  <h1>登录</h1>
  <form onsubmit={handleLogin}>
    <div>
      <label>用户名:</label>
      <input type="text" bind:value={username} />
    </div>
    <div>
      <label>密码:</label>
      <input type="password" bind:value={password} />
    </div>
    <button type="submit">登录</button>
  </form>
  <button onclick={() => router.push('qr', {}, 'slide-left')}>扫码</button>
</div>

<style>
  .sign-page {
    padding: 20px;
  }
  
  form div {
    margin-bottom: 10px;
  }
  
  label {
    display: block;
    margin-bottom: 5px;
  }
  
  input {
    width: 100%;
    padding: 8px;
    box-sizing: border-box;
  }
  
  button {
    margin-right: 10px;
    margin-top: 10px;
  }
</style>