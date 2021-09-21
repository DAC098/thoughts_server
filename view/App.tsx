import { Breadcrumb, IconButton, ScrollablePane, Stack } from "@fluentui/react"
import React, { useEffect } from "react"
import { Route, Switch, useHistory, useLocation } from "react-router-dom"

import NavSection from "./NavSection"
import EntriesView from "./routes/entries"
import CustomFieldsView from "./routes/custom_fields"
import EntryId from "./routes/entries/entry_id"
import Users from "./routes/users"
import UserIdView from "./routes/users/user_id"
import AccountView from "./routes/account"
import SettingsView from "./routes/settings"
import FieldIdView from "./routes/custom_fields/field_id"
import AdminUserListView from "./routes/admin/users"
import AdminUserIdView from "./routes/admin/users/users_id"
import TagsView from "./routes/tags"
import TagsIDView from "./routes/tags/tag_id"
import useAppDispatch from "./hooks/useAppDispatch"
import useAppSelector from "./hooks/useAppSelector"
import { view_actions } from "./redux/slices/view"
import resize_listener from "./util/ResizeListener"
import GlobalCustomFieldsView from "./routes/global/custom_fields"
import GlobalCustomFieldIdView from "./routes/global/custom_fields/field_id"

const App = () => {
    const location = useLocation();
    const history = useHistory();
    const app_dispatch = useAppDispatch();
    const view_state = useAppSelector(state => state.view);

    useEffect(() => {
        let is_min_width = window.innerWidth <= 1080;

        const onResize = (width: number, height: number) => {
            if (width <= 1080) {
                if (!is_min_width) {
                    app_dispatch(view_actions.update_visibility(true));
                    is_min_width = true;
                }
            } else {
                if (is_min_width) {
                    app_dispatch(view_actions.update_visibility(false));
                    is_min_width = false;
                }
            }
        }

        resize_listener.on("resize", onResize);

        return () => {
            resize_listener.off("resize", onResize);
        }
    }, [])

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

    return <Stack id={"main"} horizontal style={{position: "relative", width: "100vw", height: "100vh"}}>
        <Stack.Item id={"nav_section"} shrink={0} grow={0}>
            <NavSection/>
        </Stack.Item>
        <Stack.Item id={"content_and_header"} grow styles={{root: {
            maxWidth: view_state.is_min_width ? "100vw" : (
                view_state.visible ? "calc(100vw - 200px)" : "100vw"
            ),
            maxHeight: "100vh"
        }}}>
            <Stack styles={{root: {position: "relative", width: "100%", height: "100%"}}}>
                <Stack.Item id={"header"} shrink={0} grow={0} styles={{root: {backgroundColor: "black"}}}>
                    <Stack horizontal verticalAlign="center" tokens={{childrenGap: 8, padding: "4 8px"}}>
                        <IconButton
                            iconProps={{iconName: "GlobalNavButton"}} 
                            onClick={() => app_dispatch(view_actions.set_visible(!view_state.visible))}
                        />
                        <Stack.Item grow>
                            <Breadcrumb items={breadcrumb_items} styles={{root: {marginBottom: 0, marginTop: 0}}}/>
                        </Stack.Item>
                    </Stack>
                </Stack.Item>
                <Stack.Item id="content" grow styles={{root: {
                    position: "relative"
                }}}>
                    <Switch>
                        <Route path="/account" exact component={AccountView}/>
                        <Route path="/settings" exact component={SettingsView}/>
                        <Route path={["/tags", "/tags/:tag_id"]} exact children={({match}) => 
                            match ? <>
                                <TagsView/>
                                <Route path="/tags/:tag_id" exact children={({match}) => 
                                    match ? <TagsIDView/> : null
                                }/>
                            </> : null
                        }/>
                        <Route path={["/custom_fields","/custom_fields/:field_id"]} exact children={({match}) => 
                            match ? <>
                                <CustomFieldsView/>
                                <Route path="/custom_fields/:field_id" exact children={({match}) => 
                                    match ? <FieldIdView/> : null
                                }/>
                            </> : null
                        }/>
                        <Route path={["/entries", "/entries/:entry_id"]} exact children={({match}) => 
                            match ? <>
                                <EntriesView/>
                                <Route path="/entries/:entry_id" exact children={({match}) =>
                                    match ? <EntryId/> : null
                                }/>
                            </> : null
                        }/>
                        <Route path="/users" children={({match}) =>
                            match ? <Switch>
                                <Route path={["/users/:user_id/custom_fields"]} exact children={({match}) =>
                                    match ? <>
                                        <CustomFieldsView user_specific/>
                                    </> : null
                                }/>
                                <Route path={["/users/:user_id/entries", "/users/:user_id/entries/:entry_id"]} exact children={({match}) => 
                                    match ? <>
                                        <EntriesView user_specific/>
                                        <Route path="/users/:user_id/entries/:entry_id" exact children={({match}) =>
                                            match ? <EntryId/> : null
                                        }/>
                                    </> : null
                                }/>
                                <Route path={["/users/:user_id/tags"]} exact children={({match}) => 
                                    match ? <>
                                        <TagsView/>
                                    </> : null
                                }/>
                                <Route path={["/users", "/users/:user_id"]} exact children={({match}) => 
                                    match ? <>
                                        <Users/>
                                        <Route path="/users/:user_id" exact component={UserIdView}/>
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
                        <Route path={["/global/custom_fields", "/global/custom_fields/:field_id"]} exact children={({match}) =>
                            match ? <>
                                <GlobalCustomFieldsView/>
                                <Route path="/global/custom_fields/:field_id" exact children={({match}) => 
                                    match ? <GlobalCustomFieldIdView/> : null
                                }/>
                            </> : null
                        }/>
                        <Route component={() => <div>Page Not Found</div>}/>
                    </Switch>
                </Stack.Item>
            </Stack>
        </Stack.Item>
    </Stack>
}

export default App;