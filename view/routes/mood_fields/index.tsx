import { CommandBar, DetailsList, ScrollablePane, Spinner, Stack, Sticky, StickyPositionType } from "@fluentui/react";
import React, { useEffect } from "react"
import { useHistory, useParams } from "react-router";
import { useAppDispatch, useAppSelector } from "../../hooks/useApp";
import { MoodFieldJson } from "../../api/types";
import { actions as mood_fields_actions } from "../../redux/mood_fields"
import { Link } from "react-router-dom";
import { useOwner } from "../../hooks/useOwner";

interface MoodFieldsViewProps {
    user_specific?: boolean
}

const MoodFieldsView = ({user_specific = false}: MoodFieldsViewProps) => {
    const params = useParams<{user_id?: string}>();
    const history = useHistory();

    const mood_fields_state = useAppSelector(state => state.mood_fields);
    const dispatch = useAppDispatch();

    const owner = useOwner(user_specific);

    const loadFields = () => {
        if (mood_fields_state.loading) {
            return;
        }

        dispatch(mood_fields_actions.fetchMoodFields({
            owner,
            user_specific
        }));
    }

    useEffect(() => {
        if (mood_fields_state.owner !== owner) {
            loadFields();
        }
    }, [owner])

    return <Stack
        style={{
            width: "100%", height: "100%",
            position: "relative"
        }}
    >
        <ScrollablePane>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <Stack horizontal verticalAlign="center" horizontalAlign="start">
                    <Stack.Item style={{minWidth: 230}}>
                        <CommandBar items={[
                            {
                                key: "refresh",
                                text: "Refresh",
                                iconProps: {iconName: "Refresh"},
                                onClick: loadFields
                            },
                            {
                                key: "new_field",
                                text: "New Field",
                                iconProps: {iconName: "Add"},
                                onClick: () => history.push("/mood_fields/0")
                            }
                        ]}/>
                    </Stack.Item>
                    <div style={{display: mood_fields_state.loading ? null : "none"}}>
                        <Spinner label="loading" labelPosition="right"/>
                    </div>
                </Stack>
            </Sticky>
            <DetailsList
                items={mood_fields_state.mood_fields}
                onRenderDetailsHeader={(p, d) => {
                    return <Sticky stickyPosition={StickyPositionType.Header}>
                        {d(p)}
                    </Sticky>
                }}
                columns={[
                    {
                        key: "name",
                        name: "Name",
                        minWidth: 80,
                        maxWidth: 120,
                        onRender: (item: MoodFieldJson) => {
                            return <Link to={{
                                pathname: `${user_specific ? `/users/${params.user_id}` : ""}/mood_fields/${item.id}`,
                                state: {field: item}
                            }}>
                                {item.name}
                            </Link>
                        }
                    },
                    {
                        key: "type",
                        name: "Type",
                        minWidth: 80,
                        maxWidth: 120,
                        onRender: (item: MoodFieldJson) => {
                            return item.config.type
                        }
                    },
                    {
                        key: "comment",
                        name: "Comment",
                        minWidth: 80,
                        fieldName: "comment"
                    }
                ]}
            />
        </ScrollablePane>
    </Stack>
}

export default MoodFieldsView;