import { CommandBar, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType } from "@fluentui/react";
import React from "react";
import { useEffect, useState } from "react";
import { useHistory, useLocation } from "react-router"
import { Link } from "react-router-dom";
import apiv2 from "../../../apiv2";
import { GlobalCustomField } from "../../../apiv2/types";
import { stringFromLocation } from "../../../util/url";

export interface GlobalCustomFieldsViewProps {}

const GlobalCustomFieldsView = ({}: GlobalCustomFieldsViewProps) => {
    const history = useHistory();
    const location = useLocation();
    const [loading, setLoading] = useState(false);
    const [global_custom_fields, setGlobalCustomFields] = useState<GlobalCustomField[]>([]);

    const fetchGlobalCustomFields = () => {
        setLoading(true);

        apiv2.global.custom_fields.get({}).then(res => {
            setGlobalCustomFields(res.body.data);
        }).catch(console.error).then(() => {
            setLoading(false);
        })
    }

    useEffect(() => {
        fetchGlobalCustomFields();
    }, []);

    return <Stack
        style={{
            width: "100%", height: "100%",
            position: "relative"
        }}
    >
        <ScrollablePane>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <CommandBar items={[
                    {
                        key: "refresh",
                        text: "Refresh",
                        iconProps: {iconName: "Refresh"},
                        onClick: () => fetchGlobalCustomFields()
                    },
                    {
                        key: "new_field",
                        text: "New Field",
                        iconProps: {iconName: "Add"},
                        onClick: () => history.push(`/global/custom_fields/0?prev=${stringFromLocation(location, {encode: true})}`)
                    }
                ]}/>
            </Sticky>
            <ShimmeredDetailsList
                items={global_custom_fields}
                enableShimmer={loading}
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
                        onRender: (item: GlobalCustomField) => {
                            return <Link to={{
                                pathname: `/global/custom_fields/${item.id}?prev=${stringFromLocation(location, {encode: true})}`
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
                        onRender: (item: GlobalCustomField) => {
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

export default GlobalCustomFieldsView;