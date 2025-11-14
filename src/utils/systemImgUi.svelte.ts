<script lang="ts">
  import { onMount } from "svelte";
  import { loadImage } from "../lib/image"; // 统一接口
  import type { ImageResult } from "../lib/image";

  let images: ImageResult[] = [];
  let selected: ImageResult | null = null;
  let error: string | null = null;

  // 分页 / 懒加载
  let visibleImages: ImageResult[] = [];
  let pageSize = 20;
  let currentPage = 0;

  onMount(async () => {
    try {
      const results = await loadImage();
      images = Array.isArray(results) ? results : [results];
      loadNextPage();
    } catch (e: any) {
      error = e?.toString() || "Failed to load images";
    }
  });

  function loadNextPage() {
    const start = currentPage * pageSize;
    const end = start + pageSize;
    visibleImages = [...visibleImages, ...images.slice(start, end)];
    currentPage++;
  }

  function chooseImage(img: ImageResult) {
    selected = img;
  }

  function confirmSelection() {
    if (selected) {
      console.log("Selected image:", selected.path);
      // 可以触发事件或者传递给父组件
    }
  }
</script>

{#if error}
  <p class="error">{error}</p>
{:else}
  {#if images.length === 0}
    <p>No images found in Pictures directory.</p>
  {:else}
    <div class="image-grid">
      {#each visibleImages as img}
        <div
          class="image-item {selected === img ? 'selected' : ''}"
          on:click={() => chooseImage(img)}
          on:mouseover={() => previewImg = img}
        >
          <img
            src={"data:image/*;base64," + btoa(String.fromCharCode(...img.bytes.slice(0, 1024*50)))}
            alt={img.path?.split("/").pop()}
            title={img.path?.split("/").pop()}
          />
          <p>{img.path?.split("/").pop()}</p>
        </div>
      {/each}
    </div>

    {#if images.length > visibleImages.length}
      <button on:click={loadNextPage}>Load More</button>
    {/if}

    <button on:click={confirmSelection} disabled={!selected}>
      Confirm Selection
    </button>

    {#if selected}
      <div class="preview">
        <h3>Preview</h3>
        <img src={"data:image/*;base64," + btoa(String.fromCharCode(...selected.bytes))} />
      </div>
    {/if}
  {/if}
{/if}

<style>
  .image-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    max-height: 500px;
    overflow-y: auto;
  }

  .image-item {
    border: 2px solid transparent;
    padding: 4px;
    cursor: pointer;
    width: 150px;
    text-align: center;
    position: relative;
  }

  .image-item.selected {
    border-color: #007acc;
  }

  img {
    width: 100%;
    height: auto;
    display: block;
    border-radius: 4px;
  }

  button {
    margin-top: 12px;
    padding: 8px 16px;
    font-size: 1rem;
    cursor: pointer;
  }

  .error {
    color: red;
  }

  .preview {
    margin-top: 20px;
    border: 1px solid #ccc;
    padding: 12px;
    border-radius: 6px;
    max-width: 600px;
  }

  .preview img {
    width: 100%;
    height: auto;
  }
</style>
