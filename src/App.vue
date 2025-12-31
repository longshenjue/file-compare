<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import HomePage from './components/HomePage.vue';
import ConfigList from './components/ConfigList.vue';
import ConfigEditor from './components/ConfigEditor.vue';
import ReconcilePage from './components/ReconcilePage.vue';
import TaskList from './components/TaskList.vue';
import TaskDetail from './components/TaskDetail.vue';
import OrderManagement from './components/OrderManagement.vue';

const currentPage = ref<'home' | 'config-list' | 'config-edit' | 'reconcile' | 'task-list' | 'task-detail' | 'order-management'>('home');
const editingConfigId = ref<string | undefined>(undefined);
const viewingTaskId = ref<string | undefined>(undefined);

function navigate(page: string, id?: string) {
  currentPage.value = page as any;
  if (page === 'config-edit') {
    editingConfigId.value = id;
  } else if (page === 'task-detail') {
    viewingTaskId.value = id;
  }
}

// 首次启动初始化默认配置
onMounted(async () => {
  try {
    await invoke('init_default_config');
  } catch (e) {
    console.error('初始化默认配置失败:', e);
  }
});
</script>

<template>
  <div>
    <HomePage v-if="currentPage === 'home'" @navigate="navigate" />
    <ConfigList v-else-if="currentPage === 'config-list'" @navigate="navigate" />
    <ConfigEditor v-else-if="currentPage === 'config-edit'" :config-id="editingConfigId" @navigate="navigate" />
    <ReconcilePage v-else-if="currentPage === 'reconcile'" @navigate="navigate" />
    <TaskList v-else-if="currentPage === 'task-list'" @navigate="navigate" />
    <TaskDetail v-else-if="currentPage === 'task-detail'" :task-id="viewingTaskId!" @navigate="navigate" />
    <OrderManagement v-else-if="currentPage === 'order-management'" @navigate="navigate" />
  </div>
</template>