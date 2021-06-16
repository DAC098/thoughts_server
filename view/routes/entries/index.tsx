import React, { useEffect, useReducer } from "react"
import { ScrollablePane, Stack, Sticky, StickyPositionType } from "@fluentui/react"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useLoadFields } from "../../hooks/useLoadFields"
import { useOwner } from "../../hooks/useOwner"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { tags_actions } from "../../redux/slices/tags"
import { GraphView } from "./GraphView"
import { TableView } from "./TableView"
import { EntriesViewReducer, entriesViewSlice, initialState, entries_view_actions, EntriesViewContext } from "./reducer"
import { CommandBarView } from "./CommandBarView"

interface EntriesViewProps {
    user_specific?: boolean
}

const EntriesView = ({user_specific = false}: EntriesViewProps) => {
    const owner = useOwner(user_specific);
    const tags_state = useAppSelector(state => state.tags);
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const appDispatch = useAppDispatch();

    const loadEntries = useLoadEntries();
    const loadFields = useLoadFields();
    
    const [state, dispatch] = useReducer<EntriesViewReducer>(
        entriesViewSlice.reducer, 
        initialState(custom_fields_state.custom_fields)
    );
    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    useEffect(() => {
        dispatch(entries_view_actions.set_fields(custom_fields_state.custom_fields))
    }, [custom_fields_state.custom_fields])
    
    useEffect(() => {
        if (entries_state.owner !== owner) {
            loadEntries(
                owner, user_specific, 
                {
                    from: entries_state.from != null ? new Date(entries_state.from) : null, 
                    to: entries_state.to != null ? new Date(entries_state.to) : null
                }
            );
        }

        if (custom_fields_state.owner !== owner) {
            loadFields(owner, user_specific);
        }

        if (tags_state.owner !== owner) {
            appDispatch(tags_actions.fetchTags({owner, user_specific}));
        }
    }, [owner]);

    return <EntriesViewContext.Provider value={dispatch}>
        <Stack style={{
            position: "relative",
            width: "100%",
            height: "100%"
        }}>
            {state.view_graph ?
                <>
                    <CommandBarView owner={owner} user_specific={user_specific} entries_view_state={state}/>
                    {!loading_state && state.selected_field != null ?
                        <GraphView field={state.selected_field}/>
                        :
                        null
                    }
                </>
                :
                <ScrollablePane styles={{"contentContainer": {height: "100%"}}}>
                    <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                        <CommandBarView owner={owner} user_specific={user_specific} entries_view_state={state}/>
                    </Sticky>
                    <TableView user_specific={user_specific} owner={owner} visible_fields={state.visible_fields}/>
                </ScrollablePane>
            }
        </Stack>
    </EntriesViewContext.Provider>
}

export default EntriesView;