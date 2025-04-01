import type { Tab } from "@/components/tabs/model";
import { useSessionStore } from "@/stores/session";
import { ElForm, ElFormItem, ElInput, ElMessageBox } from "element-plus";
import { ref } from "vue";

const newSession = async () => {
  const sessionStore = useSessionStore();

  const id = Date.now().toString();
  const newSession = ref<Tab>({
    label: `Session ${id}`,
    id
  });

  ElMessageBox({
    title: "New Session",
    message: (
      <div>
        <p>请输入新会话的名称和Id</p>
        <ElForm label-width="auto" model={newSession}>
          <ElFormItem label="Name" prop="label">
            <ElInput
              style={{ width: "320px" }}
              type="text"
              v-model={newSession.value.label}
            />
          </ElFormItem>
          <ElFormItem label="Id" prop="id">
            <ElInput type="string" v-model={newSession.value.id} />
          </ElFormItem>
        </ElForm>
      </div>
    ),

    showCancelButton: true
  })
    .then((box) => {
      if (box === "confirm") {
        sessionStore.addSession(newSession.value);
      }
    })
    .catch(() => {});
};

export const fileMenuItems = [
  {
    label: "New Session",
    action: "new-session",
    click: newSession
  }
];
