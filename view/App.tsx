import { Breadcrumb, IconButton, ScrollablePane, Stack } from "@fluentui/react"
import React from "react"
import { Route, Switch, useHistory, useLocation } from "react-router-dom"

import NavSection from "./NavSection"
import Entries from "./routes/entries"
import MoodFieldsView from "./routes/mood_fields"
import EntryId from "./routes/entries/entry_id"
import Users from "./routes/users"
import UserId from "./routes/users/user_id"
import AccountView from "./routes/account"
import SettingsView from "./routes/settings"
import FieldIdView from "./routes/mood_fields/field_id"
import AdminUserListView from "./routes/admin/users"
import AdminUserIdView from "./routes/admin/users/users_id"

const App = () => {
    const location = useLocation();
    const history = useHistory();

    let breadcrumb_items = [];
    let previous = [];
    let split = location.pathname.split("/");
    let count = 0;

    for (let segment of split) {
        if (segment.length === 0) {
            count++;
            continue;
        }

        previous.push(segment);

        let crumb = {
            text: segment,
            key: segment
        };

        if ((count++) + 1 < split.length) {
            let path = "/" + previous.join("/");
            
            crumb["onClick"] =  () => {
                history.push(path);
            }
        }

        breadcrumb_items.push(crumb);
    }

    return <Stack horizontal style={{position: "relative", width: "100vw", height: "100vh"}}>
        <Stack.Item shrink={0} grow={0} style={{width: 180}}>
            <NavSection/>
        </Stack.Item>
        <Stack.Item grow>
            <Stack style={{position: "relative", width: "100%", height: "100%"}}>
                <Stack.Item shrink={0} grow={0} style={{backgroundColor: "black"}}>
                    <Breadcrumb items={breadcrumb_items}/>
                </Stack.Item>
                <Stack.Item id="main_content" grow style={{position: "relative"}}>
                    <Switch>
                        <Route path="/account" exact component={AccountView}/>
                        <Route path="/settings" exact component={SettingsView}/>
                        <Route path="/tags" exact component={() => <div>Tags</div>}/>
                        <Route path={["/mood_fields","/mood_fields/:field_id"]} exact children={({match}) => 
                            match ? <>
                                <MoodFieldsView/>
                                <Route path="/mood_fields/:field_id" exact children={({match}) => 
                                    match ? <FieldIdView/> : null
                                }/>
                            </> : null
                        }/>
                        <Route path={["/entries", "/entries/:entry_id"]} exact children={({match}) => 
                            match ? <>
                                <Entries/>
                                <Route path="/entries/:entry_id" exact children={({match}) =>
                                    match ? <EntryId/> : null
                                }/>
                            </> : null
                        }/>
                        <Route path="/users" children={({match}) =>
                            match ? <Switch>
                                <Route path={["/users/:user_id/mood_fields", "/users/:user_id/mood_fields/:field_id"]} exact children={({match}) =>
                                    match ? <>
                                        <MoodFieldsView user_specific/>
                                        <Route path="/users/:user_id/mood_fields/:field_id" exact children={({match}) => 
                                            match ? <FieldIdView/> : null
                                        }/>
                                    </> : null
                                }/>
                                <Route path={["/users/:user_id/entries", "/users/:user_id/entries/:entry_id"]} exact children={({match}) => 
                                    match ? <>
                                        <Entries user_specific/>
                                        <Route path="/users/:user_id/entries/:entry_id" exact children={({match}) =>
                                            match ? <EntryId user_specific/> : null
                                        }/>
                                    </> : null
                                }/>
                                <Route path={["/users", "/users/:user_id"]} exact children={({match}) => 
                                    match ? <>
                                        <Users/>
                                        <Route path="/users/:user_id" exact component={UserId}/>
                                    </> : null
                                }/>
                            </Switch> : null
                        }/>
                        <Route path="/admin" children={({match}) => 
                            match ? <Switch>
                                <Route path={["/admin/users", "/admin/users/:user_id"]} exact children={({match}) => 
                                    match ? <>
                                        <AdminUserListView/>
                                        <Route path="/admin/users/:user_id" exact children={({match}) => 
                                            match ? <AdminUserIdView/> : null
                                        }/>
                                    </> : null
                                }/>
                            </Switch> : null
                        }/>
                    </Switch>
                </Stack.Item>
            </Stack>
        </Stack.Item>
    </Stack>
}

export default App;