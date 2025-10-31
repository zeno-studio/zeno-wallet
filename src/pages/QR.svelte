<script lang="ts">
  import { router } from '../lib/runes';
  import { startScan } from '../lib/ts/scanner';
  
  let scannedData = '';
  let isScanning = false;
  
  async function handleScan() {
    try {
      isScanning = true;
      const result = await startScan();
      scannedData = result.content;
      router.push('export', { scannedData: result.content }, 'slide-left');
    } catch (error) {
      console.error('扫描失败:', error);
      alert('扫描失败: ' + error);
    } finally {
      isScanning = false;
    }
  }
</script>

<div class="qr-page">
  <h1>二维码扫描</h1>
  <button on:click={handleScan} disabled={isScanning}>
    {isScanning ? '扫描中...' : '开始扫描'}
  </button>
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