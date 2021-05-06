import React from "react"
import { render } from "react-dom"
import { BrowserRouter, Route, Switch } from "react-router-dom"
import App from "./App"

import "./request"

import { initializeIcons } from "@fluentui/react/lib/Icons"
import Login from "./routes/auth/login"
import { Provider } from "react-redux"
import { store } from "./redux/store"

initializeIcons();

document.addEventListener("DOMContentLoaded", e => {
    render(
        <Provider store={store}>
            <BrowserRouter basename="/">
                <Switch>
                    <Route path="/auth/login" exact component={Login}/>
                    <Route path="/" component={App}/>
                </Switch>
            </BrowserRouter>
        </Provider>,
        document.getElementById("render-root")
    );
}, {once: true});