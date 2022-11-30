import { createApp } from "vue";
import App from "./App.vue";
import { CacheManager } from "./model/data_manager";

async function load() {
	const designTheme = CacheManager.getDesignTheme();

	if (designTheme === "dark") {
		await import("../node_modules/ant-design-vue/dist/antd.dark.css");
	}

	createApp(App).mount("#app");
}

load();
