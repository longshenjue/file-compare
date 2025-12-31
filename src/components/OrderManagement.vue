<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, message } from '@tauri-apps/plugin-dialog';
import type { ChannelConfig, OrderFile } from '../types';

const emit = defineEmits<{
  navigate: [page: string];
}>();

const configs = ref<ChannelConfig[]>([]);
const orderFiles = ref<OrderFile[]>([]);
const selectedConfig = ref<ChannelConfig | null>(null);
const selectedSource = ref<'sourceA' | 'sourceB'>('sourceA');
const uploadDate = ref(new Date().toISOString().split('T')[0]); // 用于去重的日期
const loading = ref(false);
const uploading = ref(false);
const viewingData = ref<any[]>([]);
const showDataModal = ref(false);

// 查询条件
const queryConfigId = ref<string | null>(null);
const querySourceName = ref<string | null>(null);
const queryField = ref('');
const queryOperator = ref('equals');
const queryValue = ref('');
const queryLimit = ref(100);

// 清除历史数据
const showCleanupModal = ref(false);
const cleanupDate = ref('');
const cleanupLoading = ref(false);

const availableFields = computed(() => {
  if (viewingData.value.length === 0) return [];
  return Object.keys(viewingData.value[0] || {});
});

// 获取所有列名，按照固定顺序排序
const tableColumns = computed(() => {
  if (viewingData.value.length === 0) return [];
  
  // 收集所有行的所有键
  const allKeys = new Set<string>();
  for (const row of viewingData.value) {
    for (const key of Object.keys(row)) {
      allKeys.add(key);
    }
  }
  
  // 按照固定顺序排序
  const sortedKeys: string[] = [];
  const remainingKeys = Array.from(allKeys);
  
  // 1. ID 字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('id') && !key.includes('normalized') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 2. 时间字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('time') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 3. 日期字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('date') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 4. 金额字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('amount') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 5. 状态字段
  for (const key of remainingKeys) {
    if (key.toLowerCase().includes('status') && !sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 6. 标准化字段
  for (const key of remainingKeys) {
    if (key.startsWith('source') || key.includes('normalized') || key.includes('original')) {
      if (!sortedKeys.includes(key)) {
        sortedKeys.push(key);
      }
    }
  }
  
  // 7. 其他字段（按字母顺序）
  for (const key of remainingKeys) {
    if (!sortedKeys.includes(key)) {
      sortedKeys.push(key);
    }
  }
  
  // 最后按字母顺序排序剩余字段
  sortedKeys.sort();
  
  return sortedKeys;
});

// 格式化时间戳（仅对时间相关字段）
function formatTimestamp(value: any, fieldName?: string): string {
  // 如果字段名包含 amount、price、money 等金额相关关键词，不格式化
  if (fieldName) {
    const fieldLower = fieldName.toLowerCase();
    if (fieldLower.includes('amount') || 
        fieldLower.includes('price') || 
        fieldLower.includes('money') || 
        fieldLower.includes('cost') || 
        fieldLower.includes('fee') ||
        fieldLower.includes('total') ||
        fieldLower.includes('sum')) {
      return String(value);
    }
  }
  
  // 如果字段名不包含 time、date、timestamp 等时间相关关键词，不格式化
  if (fieldName) {
    const fieldLower = fieldName.toLowerCase();
    if (!fieldLower.includes('time') && 
        !fieldLower.includes('date') && 
        !fieldLower.includes('timestamp') &&
        !fieldLower.includes('created') &&
        !fieldLower.includes('updated') &&
        !fieldLower.includes('upload')) {
      return String(value);
    }
  }
  
  // 如果是数字类型的时间戳
  if (typeof value === 'number') {
    let timestamp = value;
    
    // 判断是微秒（16位）还是毫秒（13位）
    if (timestamp > 1000000000000000) {
      // 微秒时间戳，转换为毫秒
      timestamp = Math.floor(timestamp / 1000);
    } else if (timestamp < 1000000000) {
      // 秒时间戳，转换为毫秒
      timestamp = timestamp * 1000;
    }
    
    // 验证时间戳是否在合理范围内（1970-2100年）
    if (timestamp > 0 && timestamp < 4102444800000) {
      try {
        const date = new Date(timestamp);
        // 验证日期是否有效
        if (!isNaN(date.getTime())) {
          return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
          });
        }
      } catch (e) {
        // 日期无效，返回原值
      }
    }
  }
  
  // 如果是字符串形式的时间戳
  if (typeof value === 'string' && /^\d+$/.test(value)) {
    let timestamp = parseInt(value);
    
    // 判断是微秒（16位）还是毫秒（13位）
    if (timestamp > 1000000000000000) {
      // 微秒时间戳，转换为毫秒
      timestamp = Math.floor(timestamp / 1000);
    } else if (timestamp < 1000000000) {
      // 秒时间戳，转换为毫秒
      timestamp = timestamp * 1000;
    }
    
    // 验证时间戳是否在合理范围内（1970-2100年）
    if (timestamp > 0 && timestamp < 4102444800000) {
      try {
        const date = new Date(timestamp);
        // 验证日期是否有效
        if (!isNaN(date.getTime())) {
          return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
          });
        }
      } catch (e) {
        // 日期无效，返回原值
      }
    }
  }
  
  // 如果不是时间戳，直接返回原值
  return String(value);
}

async function loadConfigs() {
  try {
    configs.value = await invoke<ChannelConfig[]>('load_configs');
  } catch (e: any) {
    await message(`加载配置失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function loadOrderFiles() {
  loading.value = true;
  try {
    orderFiles.value = await invoke<OrderFile[]>('list_order_files');
    orderFiles.value.sort((a, b) => new Date(b.uploadTime).getTime() - new Date(a.uploadTime).getTime());
  } catch (e: any) {
    await message(`加载订单文件失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    loading.value = false;
  }
}

async function uploadFile() {
  if (!selectedConfig.value) {
    await message('请先选择配置', {
      title: '提示',
      kind: 'info',
    });
    return;
  }

  if (!uploadDate.value) {
    await message('请选择上传日期（用于去重）', {
      title: '提示',
      kind: 'info',
    });
    return;
  }

  try {
    const filePath = await open({
      title: '选择文件',
      filters: [{ name: 'CSV Files', extensions: ['csv'] }],
    });

    if (!filePath) return;

    uploading.value = true;

    const sourceName = selectedSource.value === 'sourceA'
      ? selectedConfig.value.sourceAName
      : selectedConfig.value.sourceBName;

    const mappings = selectedSource.value === 'sourceA' 
      ? selectedConfig.value.sourceAConfig.mappings
      : selectedConfig.value.sourceBConfig.mappings;

    const headerRow = selectedSource.value === 'sourceA'
      ? selectedConfig.value.sourceAConfig.header
      : selectedConfig.value.sourceBConfig.header;

    const fileName = (filePath as string).split('/').pop() || '';

    const result = await invoke<OrderFile>('upload_order_file', {
      configId: selectedConfig.value.id,
      configName: selectedConfig.value.name,
      sourceName,
      filePath,
      fileName,
      uploadDate: uploadDate.value,
      headerRow,
      mappings,
    });

    await message(`上传成功！共 ${result.recordCount} 条记录`, {
      title: '操作成功',
      kind: 'info',
    });
    await loadOrderFiles();
  } catch (e: any) {
    await message(`上传失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    uploading.value = false;
  }
}

async function viewFile(file: OrderFile) {
  queryConfigId.value = file.configId;
  querySourceName.value = file.sourceName;
  try {
    viewingData.value = await invoke<any[]>('query_orders', {
      configId: file.configId,
      sourceName: file.sourceName,
      conditions: [],
      limit: 100,
    });
    showDataModal.value = true;
  } catch (e: any) {
    await message(`加载数据失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function queryData() {
  if (!queryField.value) {
    await message('请选择查询字段', {
      title: '提示',
      kind: 'info',
    });
    return;
  }
  
  if (!queryConfigId.value || !querySourceName.value) {
    await message('请先查看某个文件以设置查询范围', {
      title: '提示',
      kind: 'info',
    });
    return;
  }

  try {
    const conditions = [{
      field: queryField.value,
      operator: queryOperator.value,
      value: queryValue.value,
    }];

    viewingData.value = await invoke<any[]>('query_orders', {
      configId: queryConfigId.value,
      sourceName: querySourceName.value,
      conditions,
      limit: queryLimit.value,
    });
  } catch (e: any) {
    await message(`查询失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function resetQuery() {
  if (!queryConfigId.value || !querySourceName.value) return;
  
  queryField.value = '';
  queryOperator.value = 'equals';
  queryValue.value = '';
  
  try {
    viewingData.value = await invoke<any[]>('query_orders', {
      configId: queryConfigId.value,
      sourceName: querySourceName.value,
      conditions: [],
      limit: 100,
    });
  } catch (e: any) {
    await message(`加载数据失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function deleteFile(file: OrderFile) {
  const confirmed = await message(`确认删除文件 "${file.fileName}"？`, {
    title: '确认删除',
    kind: 'warning',
    okLabel: '删除',
  });
  
  if (!confirmed) {
    return;
  }

  try {
    await invoke('delete_order_file', { fileId: file.fileId });
    await loadOrderFiles();
    if (queryConfigId.value === file.configId && querySourceName.value === file.sourceName) {
      showDataModal.value = false;
      queryConfigId.value = null;
      querySourceName.value = null;
    }
  } catch (e: any) {
    await message(`删除失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function clearAllOrders() {
  const confirmed = await message('确认清除所有订单数据？\n\n此操作将永久删除所有已上传的订单文件，且无法恢复！', {
    title: '确认删除',
    kind: 'warning',
    okLabel: '确认删除',
  });
  
  if (!confirmed) {
    return;
  }

  try {
    const deletedCount = await invoke<number>('clear_all_orders');
    await message(`成功清除所有订单数据，共删除 ${deletedCount} 个文件`, {
      title: '操作成功',
      kind: 'info',
    });
    await loadOrderFiles();
    if (showDataModal.value) {
      showDataModal.value = false;
      viewingData.value = [];
      queryConfigId.value = null;
      querySourceName.value = null;
    }
  } catch (e: any) {
    await message(`清除失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function cleanupOldOrders() {
  if (!cleanupDate.value) {
    await message('请选择日期', {
      title: '提示',
      kind: 'info',
    });
    return;
  }

  const confirmed = await message(`确认删除 ${cleanupDate.value} 之前的所有订单数据？\n\n此操作不可恢复！`, {
    title: '确认删除',
    kind: 'warning',
    okLabel: '确认删除',
  });
  
  if (!confirmed) {
    return;
  }

  cleanupLoading.value = true;
  try {
    const deletedCount = await invoke<number>('cleanup_orders_before_date', {
      beforeDate: cleanupDate.value,
    });

    await message(`成功删除 ${deletedCount} 个订单文件`, {
      title: '操作成功',
      kind: 'info',
    });
    showCleanupModal.value = false;
    cleanupDate.value = '';
    await loadOrderFiles();
  } catch (e: any) {
    await message(`清除失败: ${e}`, {
      title: '操作失败',
      kind: 'error',
    });
  } finally {
    cleanupLoading.value = false;
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN');
}

onMounted(() => {
  loadConfigs();
  loadOrderFiles();
});
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-green-50 p-8">
    <div class="max-w-7xl mx-auto">
      <div class="flex items-center justify-between mb-8">
        <div>
          <button
            @click="emit('navigate', 'home')"
            class="text-purple-600 hover:text-purple-800 mb-4 flex items-center gap-2"
          >
            ← 返回首页
          </button>
          <h1 class="text-3xl font-bold text-gray-800">订单管理</h1>
          <p class="text-gray-600 mt-2">上传、存储和查询订单数据</p>
        </div>
      </div>

      <!-- 上传区域 -->
      <div class="bg-white rounded-xl shadow-lg p-6 mb-6">
        <h2 class="text-xl font-bold text-gray-800 mb-4">上传新文件</h2>
        
        <div class="grid grid-cols-3 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">选择配置</label>
            <select
              v-model="selectedConfig"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            >
              <option :value="null">请选择配置</option>
              <option v-for="config in configs" :key="config.id" :value="config">
                {{ config.name }}
              </option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">数据源</label>
            <select
              v-model="selectedSource"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            >
              <option value="sourceA">数据源A</option>
              <option value="sourceB">数据源B</option>
            </select>
            <p class="text-xs text-gray-500 mt-1" v-if="selectedConfig">
              {{ selectedSource === 'sourceA' ? selectedConfig.sourceAName : selectedConfig.sourceBName }}
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">上传日期（用于去重）</label>
            <input
              v-model="uploadDate"
              type="date"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            />
            <p class="text-xs text-gray-500 mt-1">相同配置、数据源和日期的数据会自动去重</p>
          </div>

          <div class="flex items-end">
            <button
              @click="uploadFile"
              :disabled="!selectedConfig || uploading"
              class="w-full px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {{ uploading ? '上传中...' : '📁 选择并上传文件' }}
            </button>
          </div>
        </div>

        <p class="text-sm text-gray-500 mt-4">
          💡 提示：上传的文件会根据所选配置进行解析和清洗，然后保存到本地。支持后续查询和使用。
        </p>
      </div>

      <!-- 文件列表 -->
      <div class="bg-white rounded-xl shadow-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-bold text-gray-800">已保存的文件</h2>
          <div class="flex gap-2">
            <button
              @click="clearAllOrders"
              class="px-4 py-2 text-red-600 border border-red-600 rounded-lg hover:bg-red-50 transition-colors"
            >
              🗑️ 清除所有订单数据
            </button>
            <button
              @click="showCleanupModal = true"
              class="px-4 py-2 text-orange-600 border border-orange-600 rounded-lg hover:bg-orange-50 transition-colors"
            >
              🗑️ 清除历史数据
            </button>
            <button
              @click="loadOrderFiles"
              class="px-4 py-2 text-purple-600 border border-purple-600 rounded-lg hover:bg-purple-50 transition-colors"
              :disabled="loading"
            >
              {{ loading ? '加载中...' : '刷新' }}
            </button>
          </div>
        </div>

        <div v-if="orderFiles.length === 0" class="text-center py-12 text-gray-500">
          暂无文件，请先上传
        </div>

        <div v-else class="grid gap-4">
          <div
            v-for="file in orderFiles"
            :key="file.fileId"
            class="border border-gray-200 rounded-lg p-4 hover:border-purple-300 transition-colors"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-3 mb-2">
                  <h3 class="text-lg font-semibold text-gray-800">{{ file.fileName }}</h3>
                  <span
                    class="px-3 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-700"
                  >
                    {{ file.sourceName }}
                  </span>
                </div>

                <div class="text-sm text-gray-600 space-y-1">
                  <p><span class="font-medium">配置：</span>{{ file.configName }}</p>
                  <p><span class="font-medium">记录数：</span>{{ file.recordCount }}</p>
                  <p><span class="font-medium">上传时间：</span>{{ formatDate(file.uploadTime) }}</p>
                </div>
              </div>

              <div class="flex gap-2">
                <button
                  @click="viewFile(file)"
                  class="px-4 py-2 text-blue-600 border border-blue-600 rounded-lg hover:bg-blue-50 transition-colors"
                >
                  查看/查询
                </button>
                <button
                  @click="deleteFile(file)"
                  class="px-4 py-2 text-red-600 border border-red-600 rounded-lg hover:bg-red-50 transition-colors"
                >
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 数据查看/查询弹窗 -->
    <div v-if="showDataModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl max-w-7xl w-full max-h-[90vh] flex flex-col">
        <!-- 弹窗头部 -->
        <div class="p-6 border-b border-gray-200">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-2xl font-bold text-gray-800">查询结果</h2>
              <p class="text-sm text-gray-600 mt-1">
                共 {{ viewingData.length }} 条记录
                <span v-if="querySourceName"> | 数据源: {{ querySourceName }}</span>
              </p>
            </div>
            <button
              @click="showDataModal = false"
              class="text-gray-500 hover:text-gray-700 text-2xl"
            >
              ×
            </button>
          </div>

          <!-- 查询条件 -->
          <div class="mt-4">
            <p class="text-xs text-gray-500 mb-2" v-if="querySourceName">
              💡 当前查询范围：{{ querySourceName }}（全量历史数据）
            </p>
            <div class="grid grid-cols-5 gap-3">
              <select
                v-model="queryField"
                class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
              >
                <option value="">选择字段</option>
                <option v-for="field in availableFields" :key="field" :value="field">
                  {{ field }}
                </option>
              </select>

            <select
              v-model="queryOperator"
              class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            >
              <option value="equals">等于</option>
              <option value="contains">包含</option>
              <option value="gt">大于</option>
              <option value="lt">小于</option>
            </select>

            <input
              v-model="queryValue"
              type="text"
              placeholder="查询值"
              class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500"
            />

            <button
              @click="queryData"
              class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
            >
              🔍 查询
            </button>

            <button
              @click="resetQuery"
              class="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            >
              重置
            </button>
            </div>
          </div>
        </div>

        <!-- 数据表格 -->
        <div class="flex-1 overflow-auto p-6">
          <div v-if="viewingData.length === 0" class="text-center py-12 text-gray-500">
            无数据
          </div>

          <table v-else class="w-full text-sm">
            <thead class="bg-gray-50 sticky top-0">
              <tr>
                <th v-for="(key, index) in tableColumns" :key="index" class="px-4 py-3 text-left font-medium text-gray-700 border-b">
                  {{ key }}
                </th>
              </tr>
            </thead>
              <tbody>
                <tr v-for="(row, rowIndex) in viewingData" :key="rowIndex" class="border-b hover:bg-gray-50">
                  <td v-for="(key, colIndex) in tableColumns" :key="colIndex" class="px-4 py-3 text-gray-600">
                    {{ formatTimestamp(row[key], key) }}
                  </td>
                </tr>
              </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- 清除历史数据弹窗 -->
    <div v-if="showCleanupModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-2xl max-w-md w-full mx-4 p-6">
        <h2 class="text-2xl font-bold text-gray-800 mb-4">清除历史数据</h2>
        
        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            选择日期（将删除此日期之前的所有订单）
          </label>
          <input
            v-model="cleanupDate"
            type="date"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500"
          />
          <p class="text-sm text-gray-500 mt-2">
            💡 提示：选择的日期本身不会被删除，只删除此日期之前的数据
          </p>
        </div>

        <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
          <p class="text-sm text-red-700">
            ⚠️ <strong>警告：</strong>此操作将永久删除选定日期之前的所有订单文件，且无法恢复！
          </p>
        </div>

        <div class="flex gap-3">
          <button
            @click="cleanupOldOrders"
            :disabled="!cleanupDate || cleanupLoading"
            class="flex-1 px-6 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {{ cleanupLoading ? '删除中...' : '确认删除' }}
          </button>
          <button
            @click="showCleanupModal = false; cleanupDate = ''"
            :disabled="cleanupLoading"
            class="flex-1 px-6 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
          >
            取消
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

