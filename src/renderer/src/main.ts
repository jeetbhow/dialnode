import { mount } from "svelte";

import "./assets/main.css";

import App from "./App.svelte";

const urlParams = new URLSearchParams(window.location.search);
const view = urlParams.get("view");

const app = mount(App, {
  target: document.getElementById("app")!,
  props: {
    view
  }
});

export default app;
