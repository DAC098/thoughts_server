import { Stack, ScrollablePane, Sticky, StickyPositionType, CommandBar, Spinner, DetailsList, ShimmeredDetailsList } from "@fluentui/react";
import React, { useEffect, useState } from "react"
import { Link, useHistory } from "react-router-dom";
import { json } from "../../../request";

const AdminUserListView = () => {
    const history = useHistory();

    let [user_list, setUserList] = useState([]);
    let [loading, setLoading] = useState(false);

    const fetchUserList = () => {
        if (loading) {
            return;
        }

        setLoading(true);

        json.get("/admin/users").then(({body}) => {
            setUserList(body.data);
        }).catch(console.error).then(() => {
            setLoading(false);
        });
    }
    
    useEffect(() => {
        fetchUserList();
    }, []);

    return <Stack style={{
        position: "relative",
        width: "100%",
        height: "100%"
    }}>
        <ScrollablePane>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <Stack
                    horizontal
                    verticalAlign="center" 
                    horizontalAlign="start" 
                    tokens={{padding: "8px 8px 0", childrenGap: 8}}
                >
                    <CommandBar items={[
                        {
                            key: "refresh",
                            text: "Refresh",
                            iconProps: {iconName: "Refresh"},
                            onClick: fetchUserList
                        },
                        {
                            key: "new_user",
                            text: "New User",
                            iconProps: {iconName: "Add"},
                            onClick: () => {
                                history.push("/admin/users/0");
                            }
                        }
                    ]}/>
                </Stack>
            </Sticky>
            <ShimmeredDetailsList
                items={user_list}
                compact
                enableShimmer={loading}
                columns={[
                    {key: "id", name: "ID", minWidth: 80, maxWidth: 80, onRender: (item: any) => {
                        return <Link to={`/admin/users/${item.id}`}>{item.id}</Link>
                    }},
                    {key: "username", name: "Username", minWidth: 120, maxWidth: 180, onRender: (item: any) => {
                        return item.username
                    }},
                    {key: "level", name: "Level", minWidth: 80, maxWidth: 80, onRender: (item: any) => {
                        return item.level
                    }},
                    {key: "full_name", name: "Full Name", minWidth: 120, maxWidth: 180, onRender: (item: any) => {
                        return item.full_name
                    }},
                    {key: "email", name: "Email", minWidth: 120, maxWidth: 180, onRender: (item: any) => {
                        return item.email;
                    }}
                ]}
                onRenderDetailsHeader={(p,d) => {
                    return <Sticky stickyPosition={StickyPositionType.Header}>
                        {d(p)}
                    </Sticky>
                }}
            />
        </ScrollablePane>
    </Stack>
}

export default AdminUserListView;