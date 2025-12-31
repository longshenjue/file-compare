<template>
  <div class="min-h-screen bg-gradient-to-br from-purple-50 to-teal-50 py-8 px-4">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div class="flex items-center">
          <button
            @click="$emit('navigate', 'home')"
            class="mr-4 p-2 hover:bg-white rounded-lg transition-colors"
          >
            <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
          </button>
          <div>
            <h1 class="text-3xl font-bold text-gray-800">渠道配置管理</h1>
            <p class="text-gray-600 mt-1">创建、编辑和管理渠道解析配置</p>
          </div>
        </div>
        <div class="flex items-center space-x-3">
          <button
            @click="handleImportConfig"
            class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <span>导入配置</span>
          </button>
          <button
            @click="createNewConfig"
            class="px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center space-x-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            <span>新建配置</span>
          </button>
        </div>
      </div>
      
      <!-- Config List -->
      <div v-if="configs.length === 0" class="text-center py-20">
        <svg class="w-24 h-24 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <h3 class="text-xl font-semibold text-gray-600 mb-2">暂无配置</h3>
        <p class="text-gray-500 mb-6">点击"新建配置"创建您的第一个渠道配置</p>
      </div>
      
      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div
          v-for="config in configs"
          :key="config.id"
          class="bg-white rounded-xl shadow-lg p-6 hover:shadow-xl transition-all duration-300"
        >
          <!-- Config Header -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <h3 class="text-xl font-bold text-gray-800 mb-1">{{ config.name }}</h3>
              <p class="text-sm text-gray-500">数据源A: {{ config.sourceAName }} | 数据源B: {{ config.sourceBName }}</p>
            </div>
            <span
              :class="[
                'px-3 py-1 rounded-full text-xs font-medium',
                config.type === 'PAYOUT' ? 'bg-orange-100 text-orange-600' : 'bg-blue-100 text-blue-600'
              ]"
            >
              {{ config.type }}
            </span>
          </div>
          
          <!-- Config Info -->
          <div class="space-y-2 mb-4 text-sm text-gray-600">
            <div class="flex items-center">
              <svg class="w-4 h-4 mr-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              <span>数据源A字段: {{ config.sourceAConfig.mappings.length }} 个</span>
            </div>
            <div class="flex items-center">
              <svg class="w-4 h-4 mr-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
              </svg>
              <span>数据源B字段: {{ config.sourceBConfig.mappings.length }} 个</span>
            </div>
            <div class="flex items-center">
              <svg class="w-4 h-4 mr-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>更新: {{ formatDate(config.updatedAt) }}</span>
            </div>
          </div>
          
          <!-- Actions -->
          <div class="grid grid-cols-2 gap-2">
            <button
              @click="editConfig(config.id)"
              class="px-4 py-2 bg-purple-100 text-purple-600 rounded-lg hover:bg-purple-200 transition-colors flex items-center justify-center space-x-1"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
              <span>编辑</span>
            </button>
            <button
              @click="handleExportConfig(config)"
              class="px-4 py-2 bg-blue-100 text-blue-600 rounded-lg hover:bg-blue-200 transition-colors flex items-center justify-center space-x-1"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
              </svg>
              <span>导出</span>
            </button>
            <button
              @click="duplicateConfig(config)"
              class="px-4 py-2 bg-teal-100 text-teal-600 rounded-lg hover:bg-teal-200 transition-colors flex items-center justify-center space-x-1"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              <span>复制</span>
            </button>
            <button
              @click="confirmDelete(config)"
              class="px-4 py-2 bg-red-100 text-red-600 rounded-lg hover:bg-red-200 transition-colors flex items-center justify-center space-x-1"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              <span>删除</span>
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Delete Confirmation Modal -->
    <div
      v-if="deleteConfirm"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="deleteConfirm = null"
    >
      <div class="bg-white rounded-xl p-6 max-w-md w-full mx-4">
        <h3 class="text-xl font-bold text-gray-800 mb-4">确认删除</h3>
        <p class="text-gray-600 mb-6">
          确定要删除配置"<strong>{{ deleteConfirm.name }}</strong>"吗？此操作不可恢复。
        </p>
        <div class="flex space-x-3">
          <button
            @click="deleteConfirm = null"
            class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
          >
            取消
          </button>
          <button
            @click="deleteConfig"
            class="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save, message } from '@tauri-apps/plugin-dialog';
import { loadConfigsFromStorage, deleteConfigFromStorage, saveConfigToStorage, exportConfig, importConfig, type ChannelConfig } from '../types';

const emit = defineEmits<{
  navigate: [page: string, configId?: string];
}>();

const configs = ref<ChannelConfig[]>([]);
const deleteConfirm = ref<ChannelConfig | null>(null);

onMounted(() => {
  loadConfigs();
});

async function loadConfigs() {
  configs.value = await loadConfigsFromStorage();
  
  // 如果没有配置，自动创建默认配置
  if (configs.value.length === 0) {
    try {
      await invoke('init_default_config');
      // 重新加载配置
      configs.value = await loadConfigsFromStorage();
    } catch (e) {
      console.error('创建默认配置失败:', e);
    }
  }
}

function createNewConfig() {
  emit('navigate', 'config-edit');
}

function editConfig(id: string) {
  emit('navigate', 'config-edit', id);
}

async function duplicateConfig(config: ChannelConfig) {
  const newConfig: ChannelConfig = {
    ...config,
    id: `config-${Date.now()}`,
    name: `${config.name} (副本)`,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
  await saveConfigToStorage(newConfig);
  await loadConfigs();
}

function confirmDelete(config: ChannelConfig) {
  deleteConfirm.value = config;
}

async function deleteConfig() {
  if (deleteConfirm.value) {
    await deleteConfigFromStorage(deleteConfirm.value.id);
    await loadConfigs();
    deleteConfirm.value = null;
  }
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN');
}

async function handleExportConfig(config: ChannelConfig) {
  try {
    const filePath = await save({
      defaultPath: `${config.name}.json`,
      filters: [{ name: 'JSON配置文件', extensions: ['json'] }]
    });
    
    if (filePath) {
      await exportConfig(config, filePath);
      await message('配置导出成功！', {
        title: '操作成功',
        kind: 'info',
      });
    }
  } catch (error) {
    console.error('导出配置失败:', error);
    await message('导出配置失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  }
}

async function handleImportConfig() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON配置文件', extensions: ['json'] }]
    });
    
    if (selected && typeof selected === 'string') {
      const config = await importConfig(selected);
      await message(`配置"${config.name}"导入成功！`, {
        title: '操作成功',
        kind: 'info',
      });
      await loadConfigs();
    }
  } catch (error) {
    console.error('导入配置失败:', error);
    await message('导入配置失败: ' + error, {
      title: '操作失败',
      kind: 'error',
    });
  }
}
</script>

