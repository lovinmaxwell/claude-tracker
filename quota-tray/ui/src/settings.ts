import "./app.css";
import { mount } from "svelte";
import { initTheme } from "./lib/theme";
import SettingsApp from "./SettingsApp.svelte";

initTheme();
mount(SettingsApp, { target: document.getElementById("app")! });
