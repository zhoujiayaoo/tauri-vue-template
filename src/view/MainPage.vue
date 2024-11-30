<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { createDiscreteApi } from "naive-ui"
const { message } = createDiscreteApi(["message"])

// defineProps<{ msg: string }>()

// const projectPath = ref("")
// const javaFilePath = ref("com/fantechs/homepage/service/impl/WeaveLinkServiceImpl.java")
const javaFilePath = ref("")
const javaProcessPid = ref(null)
const executeMatchClassResult = ref({
  parent_jar_file_name: "",
  jar_file_name: "",
  class_file_name: "",
  class_file_path: "",
  output_class_path: "",
  java_process_list: []
})
const editConfigModal = ref(false)
const configJsonStr = ref("")

// const editConfig = ref({
//   projectPath: 'E:\\code\_myproject\\weave\\weave-java\\weave-server\\target',
//   serverIp: '192.168.1.201',
//   serverUsername: 'root',
//   serverPassword: '123456'
// })


async function editConfigHandle() {
  // 读取配置
  configJsonStr.value = await invoke('read_config_event', {})
  editConfigModal.value = true

}

// 保存配置按钮点击
async function editSaveHandle() {
  editConfigModal.value = false
  const result = await invoke('edit_save_handle', { config: configJsonStr.value })
  console.log(result)
  if (result == 'success') {
    message.success("保存成功")
  }
}


// 匹配class按钮点击
async function matchClassBtnHandle() {

  // await invoke('match_class_handle', { config: configJsonStr.value })
  const result: string = await invoke('match_class_handle', { javaFilePath: javaFilePath.value })
  const jsonObject = JSON.parse(result)
  console.log("result: ", result);
  console.log("jsonObject: ", jsonObject);
  executeMatchClassResult.value.parent_jar_file_name = jsonObject[0].parent_jar_file_name
  executeMatchClassResult.value.jar_file_name = jsonObject[0].jar_file_name
  executeMatchClassResult.value.class_file_name = jsonObject[0].class_file_name
  executeMatchClassResult.value.class_file_path = jsonObject[0].class_file_path
  executeMatchClassResult.value.output_class_path = jsonObject[0].output_class_path
  console.log("jsonObject: ", jsonObject);

  const java_process_list_str: string = await invoke('get_java_process_list_handle', { })
  console.log(java_process_list_str)
  const java_process_list_obj = JSON.parse(java_process_list_str)
    const formattedList = java_process_list_obj.map( (item:any) => {
      // 对齐 PID 至少 5 位宽，不足部分用空格填充
      const pidPadded = item.java_pid.padEnd(5, ' ');
      return {value: item.java_pid, label: `${pidPadded} - ${item.java_name}`}
  });

  console.log(formattedList)
  executeMatchClassResult.value.java_process_list = formattedList
  message.success("匹配完成")

}

// 开始热更新
async function startHotUpdateHandle() {
  const result: string = await invoke('start_hot_update_handle', { className: executeMatchClassResult.value.class_file_name, javaProcessPid: javaProcessPid.value })
  console.log("开始热更新结果：{}", result)
  message.success("更新完成")
}


const count = ref(0)
// async function backendAdd() {
//   count.value = await invoke('backend_add', { number: count.value })
// }
</script>

<template>
  <div>
    <n-card title="">
      <div>

        <n-gradient-text type="danger" style="margin-top: -20px;">
          <h1>poodle热更新助手</h1>
        </n-gradient-text>
        <div style="float: right;">
          <n-button type="primary" style="margin-left: 20px;" @click="editConfigHandle">编辑配置</n-button>
        </div>
      </div>
    </n-card>

    <n-card title="">
      <div style="display: flex;justify-content: space-between; ">
        <div style="flex-grow: 1;">
          <n-input v-model:value="javaFilePath" type="text" placeholder="java文件路径" />
        </div>
        <div style="width: 100px; margin-left: 10px; ">
          <n-button type="info" @click="matchClassBtnHandle">匹配class</n-button>
        </div>
      </div>
    </n-card>


    <n-card title="" v-if="executeMatchClassResult.parent_jar_file_name != ''">
      <n-alert title="匹配结果" type="info">
        <n-list bordered>
          <n-list-item>
            <n-tag :bordered="false" type="info">
              jar包名称:
            </n-tag>

            {{ executeMatchClassResult.parent_jar_file_name }}
          </n-list-item>

          <n-list-item>
            <n-tag :bordered="false" type="info">
              详细jar包路径:
            </n-tag>
            {{ executeMatchClassResult.jar_file_name }}
          </n-list-item>

          <n-list-item>
            <n-tag :bordered="false" type="info">
              class路径:
            </n-tag>
            {{ executeMatchClassResult.class_file_path }}
          </n-list-item>

          <n-list-item>
            <n-tag :bordered="false" type="info">
              class名称:
            </n-tag>
            {{ executeMatchClassResult.class_file_name }}
          </n-list-item>

          <n-list-item>
            <n-tag :bordered="false" type="success">
              java进程列表:
            </n-tag>
            <n-select style="margin-top: 5px;"  v-model:value="javaProcessPid" :options="executeMatchClassResult.java_process_list" />

          </n-list-item>



          <!-- <n-list-item>
            <n-tag :bordered="false" type="info">
              本地class路径:
            </n-tag>
            {{ executeMatchClassResult.output_class_path }}
          </n-list-item> -->

          <n-list-item>
            <n-button type="info" style="width: 100%;" @click="startHotUpdateHandle">更新</n-button>
          </n-list-item>
        </n-list>

      </n-alert>
    </n-card>

    <n-modal v-model:show="editConfigModal" class="custom-card" preset="card" title="编辑配置" style="width: 90%"
      :bordered="false">
      <template #header-extra>
      </template>


      <n-form-item label="">
        <n-input rows="10" type="textarea" v-model:value="configJsonStr" placeholder="配置信息" />
      </n-form-item>

      <template #footer>
        <n-button style="margin-top: 20px; float:right;" type="info" @click="editSaveHandle">保存配置</n-button>
      </template>
    </n-modal>


  </div>
</template>

<style scoped></style>