import "./app.css";
import { mount } from "svelte";
import SettingsApp from "./SettingsApp.svelte";

mount(SettingsApp, { target: document.getElementById("app")! });
