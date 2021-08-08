import { CommandBar, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType } from "@fluentui/react"
import React, { useEffect } from "react"
import { useHistory } from "react-router";
import { Link } from "react-router-dom";
import { Tag } from "../api/types";
import ColorSwatch from "../components/ColorSwatch";
import { useAppDispatch, useAppSelector } from "../hooks/useApp";
import { useOwner } from "../hooks/useOwner";
import { tags_actions } from "../redux/slices/tags";
import { getBrightness, min_brightness } from "../util/colors";

interface TagsViewProps {
    user_specific?: boolean
}

const TagsView = ({user_specific = false}: TagsViewProps) => {
    const color_preview_size = 20;
    const history = useHistory();
    const owner = useOwner(user_specific);
    const tags_state = useAppSelector(state => state.tags);
    const app_dispatch = useAppDispatch();

    const fetchTags = () => {
        if (tags_state.loading) {
            return;
        }

        app_dispatch(tags_actions.fetchTags({owner, user_specific}));
    }

    useEffect(() => {
        if (tags_state.owner !== owner) {
            fetchTags();
        }
    }, [owner]);

    let command_bar_actions = [
        {
            key: "refresh",
            text: "Refresh",
            iconProps: {iconName: "Refresh"},
            onClick: fetchTags
        },
        {
            key: "new_tag",
            text: "New Tag",
            iconProps: {iconName: "Add"},
            onClick: () => {
                history.push("/tags/0");
            }
        }
    ];
    
    return <Stack styles={{root: {
        position: "relative",
        width: "100%",
        height: "100%"
    }}}>
        <ScrollablePane>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <CommandBar items={command_bar_actions}/>
            </Sticky>
            <ShimmeredDetailsList
                items={tags_state.loading ? [] : tags_state.tags}
                enableShimmer={tags_state.loading}
                columns={[
                    {
                        key: "title",
                        name: "Title",
                        minWidth: 100,
                        maxWidth: 150,
                        onRender: (item: Tag) => {
                            return user_specific ?
                                item.title :
                                <Link to={`/tags/${item.id}`}>{item.title}</Link>;
                        }
                    },
                    {
                        key: "color",
                        name: "Color",
                        minWidth: 100,
                        maxWidth: 100,
                        onRender: (item: Tag) => {
                            return <Stack horizontal tokens={{childrenGap: 8}} verticalAlign="center">
                                <ColorSwatch color={item.color} borderWidth={2}/>
                                <span>{item.color}</span>
                            </Stack>;
                        }
                    },
                    {
                        key: "comment",
                        name: "Comment",
                        minWidth: 150,
                        onRender: (item: Tag) => {
                            return item.comment;
                        }
                    }
                ]}
            />
        </ScrollablePane>
    </Stack>
}

export default TagsView;