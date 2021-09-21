import React, { useEffect, useReducer } from "react"
import { ScrollablePane, Stack, Sticky, StickyPositionType } from "@fluentui/react"
import { useParams } from "react-router-dom"
import useAppSelector from "../../hooks/useAppSelector"
import { GraphView } from "./GraphView"
import { TableView } from "./TableView"
import { EntriesViewReducer, entriesViewSlice, initialState, entries_view_actions, EntriesViewContext } from "./reducer"
import { CommandBarView } from "./CommandBarView"
import { useUserId } from "../../hooks/useUserId"

interface EntriesViewProps {
    user_specific?: boolean
}

const EntriesView = ({user_specific = false}: EntriesViewProps) => {
    const user_id = useUserId();
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
                <CommandBarView user_id={user_id} entries_view_state={state}/>
            </Stack.Item>
            <Stack.Item id={"entries_data"} grow styles={{root: {position: "relative", overflow: state.view_graph ? "hidden" : null}}}>
                {state.view_graph ?
                    !loading_state && state.selected_field != null ?
                        <GraphView owner={user_id} user_specific={user_specific} field={state.selected_field}/>
                        :
                        null
                    :
                    <ScrollablePane>
                        <TableView user_specific={user_specific} owner={user_id} visible_fields={state.visible_fields}/>
                    </ScrollablePane>
                }
            </Stack.Item>
        </Stack>
    </EntriesViewContext.Provider>
}

export default EntriesView;