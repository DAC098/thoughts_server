import { CommandBar, DetailsList, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType } from "@fluentui/react";
import React, { useEffect } from "react"
import { useHistory, useParams } from "react-router";
import { useAppDispatch, useAppSelector } from "../../hooks/useApp";
import { CustomFieldJson } from "../../api/types";
import { custom_field_actions } from "../../redux/slices/custom_fields"
import { Link } from "react-router-dom";
import { useOwner } from "../../hooks/useOwner";

interface CustomFieldsViewProps {
    user_specific?: boolean
}

const CustomFieldsView = ({user_specific = false}: CustomFieldsViewProps) => {
    const params = useParams<{user_id?: string}>();
    const history = useHistory();

    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const dispatch = useAppDispatch();

    const owner = useOwner(user_specific);

    const loadFields = () => {
        if (custom_fields_state.loading) {
            return;
        }

        dispatch(custom_field_actions.fetchMoodFields({
            owner,
            user_specific
        }));
    }

    useEffect(() => {
        if (custom_fields_state.owner !== owner) {
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
                            onClick: () => history.push("/custom_fields/0")
                        }
                    ]}/>
                </Stack>
            </Sticky>
            <ShimmeredDetailsList
                items={custom_fields_state.custom_fields}
                enableShimmer={custom_fields_state.loading}
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
                        onRender: (item: CustomFieldJson) => {
                            return <Link to={{
                                pathname: `${user_specific ? `/users/${params.user_id}` : ""}/custom_fields/${item.id}`,
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
                        onRender: (item: CustomFieldJson) => {
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

export default CustomFieldsView;