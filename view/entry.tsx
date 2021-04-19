import React from "react"
import { render } from "react-dom"
import { BrowserRouter } from "react-router-dom"
import App from "./App"

import "./request"

import { initializeIcons } from "@fluentui/react/lib/Icons"

initializeIcons();

document.addEventListener("DOMContentLoaded", e => {
    render(
        <BrowserRouter basename="/">
            <App/>
        </BrowserRouter>,
        document.getElementById("render-root")
    );
}, {once: true});