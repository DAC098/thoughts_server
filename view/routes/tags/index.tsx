import { CommandBar, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType } from "@fluentui/react"
import React, { useEffect } from "react"
import { useHistory } from "react-router";
import { Link } from "react-router-dom";
import { Tag } from "../../apiv2/types";
import ColorSwatch from "../../components/ColorSwatch";
import useAppDispatch from "../../hooks/useAppDispatch"
import useAppSelector from "../../hooks/useAppSelector"
import { useGlobalFetchTags } from "../../hooks/useGlobalFetchTags";
import { useUserId } from "../../hooks/useUserId";
import { tags_actions } from "../../redux/slices/tags";
import { getBrightness, min_brightness } from "../../util/colors";

interface TagsViewProps {}

const TagsView = ({}: TagsViewProps) => {
    const history = useHistory();
    const user_id = useUserId();
    const tags_state = useAppSelector(state => state.tags);
    const globalFetchTags = useGlobalFetchTags();

    useEffect(() => {
        if (tags_state.owner !== user_id) {
            globalFetchTags({});
        }
    }, [user_id]);

    let command_bar_actions = [
        {
            key: "refresh",
            text: "Refresh",
            iconProps: {iconName: "Refresh"},
            onClick: () => globalFetchTags({})
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
                            return user_id ?
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