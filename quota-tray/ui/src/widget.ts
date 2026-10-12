import "./app.css";
import { mount } from "svelte";
import { initPalette, initTheme } from "./lib/theme";
import WidgetApp from "./WidgetApp.svelte";

initTheme();
initPalette();
mount(WidgetApp, { target: document.getElementById("app")! });
