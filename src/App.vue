<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
// import HelloWorld from './components/HelloWorld.vue'
import MainPage from './view/MainPage.vue'
import { NMessageProvider } from "naive-ui";
</script>

<template>
  <div class="flex flex-col p-2">
    <!-- <div class="flex flex-row items-center justify-center">
      <img src="/tauri.svg" class="w-20 h-20 logo tauri" alt="Tauri logo" />
      <img src="/vue.svg" class="w-20 h-20 logo vue" alt="Vue logo" />
    </div> -->
    <n-message-provider>
      <MessageApi />
    </n-message-provider>


    <MainPage></MainPage>
  </div>
</template>
