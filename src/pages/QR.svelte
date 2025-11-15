<script lang="ts">
  import { router } from '../states';
  
  let scannedData = '';
  let isScanning = false;
  
  
</script>

<div class="qr-page">
  <h1>二维码扫描</h1>

  {#if scannedData}
    <div>
      <h2>扫描结果:</h2>
      <p>{scannedData}</p>
    </div>
  {/if}
  <button on:click={() => router.pop('slide-right')}>返回</button>
</div>

<style>
  .qr-page {
    padding: 20px;
  }
  
  button {
    margin-right: 10px;
    margin-top: 10px;
  }
  
  div h2 {
    margin-top: 20px;
  }
  
  p {
    word-break: break-all;
    background: #f5f5f5;
    padding: 10px;
    border-radius: 4px;
  }
</style>