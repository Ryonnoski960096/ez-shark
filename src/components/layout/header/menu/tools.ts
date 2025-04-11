import { windowManager } from "@/stores/WindowManager";

const goToMapLocal = () => {
  windowManager.createWindow(
    {
      url: "/mapLocal"
    },
    {
      width: 800,
      height: 600,
      title: "Map Local"
    }
  );
};

export const toolsMenuItems = [
  { label: "Map Local", action: "map local", click: goToMapLocal }
];
