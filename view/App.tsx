import { Stack } from "@fluentui/react"
import React from "react"
import { Route, Switch } from "react-router-dom"

import Entries from "./routes/entries"
import Login from "./routes/auth/login"
import MoodFieldsView from "./routes/mood_fields"

const App = () => {
    return <Stack styles={{root: {height: "100vh"}}}>
        <Stack.Item grow styles={{root: {overflow: "auto"}}}>
            <Switch>
                <Route path="/auth/login" exact component={Login}/>
                <Route path="/entries" component={Entries}/>
                <Route path="/mood_fields" component={MoodFieldsView}/>
            </Switch>
        </Stack.Item>
    </Stack>
}

export default App;