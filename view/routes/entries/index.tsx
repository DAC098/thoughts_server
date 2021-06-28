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
    
    const [state, dispatch] = useReducer<EntriesViewReducer>(
        entriesViewSlice.reducer, 
        initialState(custom_fields_state.custom_fields)
    );
    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    useEffect(() => {
        dispatch(entries_view_actions.set_fields(custom_fields_state.custom_fields))
    }, [custom_fields_state.custom_fields]);

    return <EntriesViewContext.Provider value={dispatch}>
        <Stack style={{
            position: "absolute",
            width: "100%",
            height: "100%"
        }}>
            <Stack.Item id={"entries_command_bar"}>
                <CommandBarView owner={owner} user_specific={user_specific} entries_view_state={state}/>
            </Stack.Item>
            <Stack.Item id={"entries_data"} grow styles={{root: {position: "relative", overflow: state.view_graph ? "hidden" : null}}}>
                {state.view_graph ?
                    !loading_state && state.selected_field != null ?
                        <GraphView owner={owner} user_specific={user_specific} field={state.selected_field}/>
                        :
                        null
                    :
                    <ScrollablePane>
                        <TableView user_specific={user_specific} owner={owner} visible_fields={state.visible_fields}/>
                    </ScrollablePane>
                }
            </Stack.Item>
        </Stack>
    </EntriesViewContext.Provider>
}

export default EntriesView;