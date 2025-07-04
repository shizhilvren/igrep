<script setup lang="ts">
import { ref } from 'vue';
// import igrep_init, { Engine } from "igrep_pkg";
import test from '@../pkg/igrep.js'

defineProps<{
  msg_pass: string
}>()

const init_finish = ref(false);
const msg = ref('');
const engine = ref<Engine | null>(null);
const count = ref(0);
const data_file_path = "index/igrep.dat";
const index_file_path = "index/igrep.idx";

// 在组件挂载时初始化WebAssembly
import { onMounted } from 'vue';

onMounted(async () => {
  await rust_api_init();
  console.log("Rust WASM module initialized");
});

// Rust API初始化函数
async function rust_api_init() {
  try {
    // 从URL创建正确的路径
    const indexUrl = new URL(index_file_path, import.meta.url);
    let p1 = fetchFileToUint8Array(indexUrl);
    let p2 = igrep_init();
    
    let [index_file_data, _] = await Promise.allSettled([p1, p2]);

    // 检查index_file_data是否成功获取
    if (index_file_data.status === 'fulfilled' && index_file_data.value) {
      // 创建Engine实例
      engine.value = await new Engine(index_file_data.value);
      init_finish.value = true;
      console.log("Engine init finished");
    } else {
      // 类型断言，因为我们知道如果status不是fulfilled，就是rejected
      const rejected = index_file_data as PromiseRejectedResult;
      throw new Error('Failed to load index data: ' + 
          (rejected.reason instanceof Error ? rejected.reason.message : 'Unknown error'));
    }
  } catch (error) {
    console.error('Engine initialization error:', error);
    alert(`Failed to initialize search engine: ${error instanceof Error ? error.message : String(error)}`);
  }
}

async function fetchFileToUint8Array(url: URL): Promise<Uint8Array> {
  try {
    // 发起网络请求
    const response = await fetch(url);

    // 检查响应状态
    if (!response.ok) {
      throw new Error(`HTTP error! Status: ${response.status}`);
    }

    // 获取 ArrayBuffer
    const arrayBuffer = await response.arrayBuffer();

    // 转换为 Uint8Array (相当于 Rust 的 Vec<u8>)
    return new Uint8Array(arrayBuffer);
  } catch (error) {
    console.error('Error fetching file:', error);
    // 确保重新抛出的是Error对象
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('Unknown error during file fetch');
  }
}

/**
 * 从网络上读取文件的特定范围，支持u64大小的文件
 * @param url - 文件的URL
 * @param start - 起始字节位置（包含），支持u64大小的值
 * @param len - 要读取的字节长度，支持u64大小的值
 * @returns 包含请求范围内数据的Uint8Array
 */
async function fetchFileRange(url: URL, start: bigint, len: bigint): Promise<Uint8Array> {
  try {
    // 确保使用BigInt处理，以支持超过2^53-1的值
    const startBig = BigInt(start);
    const lenBig = BigInt(len);
    const endBig = startBig + lenBig - 1n; // 计算结束位置

    // 创建带有Range头的请求
    const headers = new Headers();
    headers.append('Range', `bytes=${startBig.toString()}-${endBig.toString()}`);

    // 发起网络请求
    const response = await fetch(url, { headers });

    // 检查响应状态
    if (!response.ok && response.status !== 206) {
      // 206是部分内容的HTTP状态码
      throw new Error(`HTTP error! Status: ${response.status}`);
    }

    // 获取ArrayBuffer
    const arrayBuffer = await response.arrayBuffer();

    // 转换为Uint8Array
    return new Uint8Array(arrayBuffer);
  } catch (error) {
    console.error('Error fetching file range:', error);
    console.error(`Failed range request: ${url} from ${start} length ${len}`);
    // 确保重新抛出的是Error对象
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('Unknown error during file range fetch');
  }
}

// 搜索功能实现
async function search(query: string) {
  console.log(`searching "${query}!"`)
  try {
    // 检查engine是否已初始化
    if (!engine.value) {
      alert('Search engine is not initialized');
      return;
    }
    
    // 安全地调用engine的方法
    const result = await engine.value.regex(query);
    
    // 处理搜索结果
    if (result) {
      console.log("Search result:", result);
      
      if (typeof result.len === 'function') {
        const len = result.len();
        console.log(`Found ${len} results`);
        
        if (len > 0) {
          for (let i = 0; i < len; i++) {
            const item = result.get(i);
            console.log(`Result ${i}: file ${item.file_id}, line ${item.line_id?.line_number()}`);
          }
        } else {
          console.log("No matches found");
        }
      } else if (result.all && typeof result.all === 'function' && result.all()) {
        alert("This regex is smaller than 3 characters");
      }
    }
  } catch (error) {
    console.error('Search error:', error);
    alert(`Search error: ${error instanceof Error ? error.message : String(error)}`);
  }
}
</script>

<template>
  <div>
    <div class="status-bar" :class="{ 'status-ready': init_finish }">
      WebAssembly Status: {{ init_finish ? 'Ready' : 'Initializing...' }}
    </div>
    
    <div class="search-container">
      <p>Search Regex: {{ msg }}</p>
      <div class="input-group">
        <input 
          v-model="msg" 
          placeholder="Enter regex pattern" 
          class="form-control" 
          type="text" 
          :disabled="!init_finish" 
        />
        <button 
          @click="search(msg)" 
          type="submit" 
          class="btn btn-primary"
          :disabled="!init_finish || !msg.trim()"
        >
          Search
        </button>
      </div>
      <p v-if="!init_finish" class="hint">Please wait, initializing search engine...</p>
    </div>
  </div>
  <!-- <div class="greetings">
    <h1 class="green">{{ msg_pass }}</h1>
    <h3>
      You’ve successfully created a project with
      <a href="https://vite.dev/" target="_blank" rel="noopener">Vite</a> +
      <a href="https://vuejs.org/" target="_blank" rel="noopener">Vue 3</a>.
    </h3>
  </div> -->
</template>

<style scoped>
.status-bar {
  padding: 8px 12px;
  background-color: #f8d7da;
  color: #721c24;
  border-radius: 4px;
  margin-bottom: 20px;
  text-align: center;
  transition: all 0.3s ease;
}

.status-ready {
  background-color: #d4edda;
  color: #155724;
}

.search-container {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
  background: #f9f9f9;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.input-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.form-control {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 16px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 600;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background-color: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #0069d9;
}

.hint {
  font-size: 14px;
  color: #6c757d;
  font-style: italic;
}

h1 {
  font-weight: 500;
  font-size: 2rem;
  margin-bottom: 20px;
}

@media (max-width: 640px) {
  .input-group {
    flex-direction: column;
  }
}
</style>
