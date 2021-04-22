import { Stack } from "@fluentui/react"
import React, { useEffect, useState } from "react"
import { Route, Switch, useLocation } from "react-router-dom"

import Entries from "./routes/entries"
import MoodFieldsView from "./routes/mood_fields"
import EntryId from "./routes/entries/entry_id"

const App = () => {    
    return <Stack style={{position: "relative", width: "100vw", height: "100vh"}}>
        <Stack.Item shrink={0} grow={0} style={{height: 40, backgroundColor: "black"}}>
            <div></div>
        </Stack.Item>
        <Stack.Item grow style={{position: "relative", overflow: "auto"}}>
            <Route path="/entries" exact children={({match}) => {
                return <div style={{
                    position: match ? "relative" : "absolute",
                    top: 0,
                    width: "100%",
                    height: "100%",
                    overflow: match ? "auto" : "hidden",
                    zIndex: match ? null : -1
                }}>
                    <Entries/>
                </div>
            }}/>
            <Switch>
                <Route path="/entries/:entry_id" exact component={EntryId}/>
                <Route path="/mood_fields" exact component={MoodFieldsView}/>
            </Switch>
        </Stack.Item>
    </Stack>
}

export default App;