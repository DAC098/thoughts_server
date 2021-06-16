import React, { useMemo } from "react"
import { IColumn, Icon, ShimmeredDetailsList, Sticky, StickyPositionType, TooltipHost, TooltipOverflowMode } from "@fluentui/react"
import { Link } from "react-router-dom"
import { EntryJson } from "../../api/types"
import { useAppSelector } from "../../hooks/useApp"
import TagToken from "../../components/tags/TagItem"
import { CustomFieldEntryCell } from "../../components/CustomFieldEntryCell"

export interface TableViewProps {
    user_specific: boolean
    owner: number
    visible_fields: Record<string, boolean>
}

export const TableView = ({user_specific, owner, visible_fields}: TableViewProps) => {
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const entries_state = useAppSelector(state => state.entries);
    const tags_state = useAppSelector(state => state.tags);

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    let columns = useMemo(() => {
        let rtn: IColumn[] = [
            {
                key: "date",
                name: "Date",
                minWidth: 80,
                maxWidth: 160,
                onRender: (item: EntryJson) => {
                    return <Link to={{
                        pathname: `${user_specific ? `/users/${owner}` : ""}/entries/${item.id}`
                    }}>
                        {(new Date(item.created)).toDateString()}
                    </Link>
                }
            }
        ];

        for (let field of custom_fields_state.custom_fields) {
            if (!visible_fields[field.name]) {
                continue;
            }

            rtn.push({
                key: field.name,
                name: field.name,
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    let custom_field_entry = item.custom_field_entries?.[field.id];

                    if (!custom_field_entry) {
                        return <span/>
                    }

                    let content = <>
                        <CustomFieldEntryCell value={custom_field_entry.value} config={field.config}/>
                        {custom_field_entry.comment && custom_field_entry.comment.length > 0 ?
                            <Icon style={{paddingLeft: 4}} iconName="Info"/>
                            :
                            null
                        }
                    </>;

                    return <TooltipHost overflowMode={TooltipOverflowMode.Parent} content={content}>
                        {content}
                    </TooltipHost>
                }
            })
        }

        if (tags_state.tags.length > 0) {
            rtn.push({
                key: "tags",
                name: "Tags",
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    let content = [];
    
                    for (let tag of item.tags) {
                        let title = tags_state.mapping[tag].title;
                        let color = tags_state.mapping[tag].color;
    
                        content.push(<TagToken 
                            key={tag} color={color} title={title} 
                            fontSize={null} lineHeight={20}
                        />);
                    }
    
                    return <TooltipHost overflowMode={TooltipOverflowMode.Parent} content={content} children={content}/>
                }
            })
        }

        return rtn;
    }, [loading_state, visible_fields]);

    return <ShimmeredDetailsList
        items={loading_state ? [] : entries_state.entries}
        columns={columns}
        compact={false}
        enableShimmer={loading_state}
        onRenderDetailsHeader={(p,d) => {
            return <Sticky stickyPosition={StickyPositionType.Header}>
                {d(p)}
            </Sticky>
        }}
    />
}