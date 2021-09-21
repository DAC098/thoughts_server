import { DefaultButton, IconButton, Persona, PersonaSize, Stack } from "@fluentui/react"
import React, { useEffect, useState } from "react"
import { useHistory, useLocation, useParams } from "react-router"
import { Link } from "react-router-dom"
import api from "../../apiv2"
import { User } from "../../apiv2/types"

interface UserInformationViewProps {
    user: User
}

const UserInformationView = ({user}: UserInformationViewProps) => {
    return <Stack>
        <Stack horizontal>
            <Persona
                size={PersonaSize.size120}
                text={user.full_name ?? user.username}
                secondaryText={user.full_name != null ? user.username : null}
            />
            <Stack.Item>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Link to={`/users/${user.id}/entries`}>
                        <DefaultButton text="Entries"/>
                    </Link>
                    <Link to={`/users/${user.id}/custom_fields`}>
                        <DefaultButton text="Fields"/>
                    </Link>
                    <Link to={`/users/${user.id}/tags`}>
                        <DefaultButton text="Tags"/>
                    </Link>
                </Stack>
            </Stack.Item>
        </Stack>
    </Stack>
}

const UserIdView = () => {
    const params = useParams<{user_id: string}>();
    const history = useHistory();
    const location = useLocation();

    let [loading, setLoading] = useState(false);
    let [user, setUser] = useState<User>(null);
    
    const fetchUser = () => {
        if (loading) {
            return;
        }

        setLoading(true);

        api.users.id.get({id: params.user_id}).then(res => {
            setUser(res.body.data);
        }).catch(console.error).then(() => {
            setLoading(false);
        });
    }

    useEffect(() => {
        let user_id = parseInt(params.user_id);

        if (isNaN(user_id)) {
            return;
        }

        fetchUser();
    }, [params.user_id]);

    return <Stack
        horizontal
        verticalAlign="center"
        horizontalAlign="center"
        style={{
            width: "100%", height: "100%",
            backgroundColor: "rgba(0,0,0,0.5)",
            position: "absolute",
            top: 0,
            zIndex: 1
        }}
    >
        <Stack
            style={{
                position: "relative",
                width: 600, height: "100%",
                backgroundColor: "white"
            }}
            tokens={{padding: 8}}
        >
            <IconButton 
                iconProps={{iconName: "Cancel"}} 
                style={{position: "absolute", top: 0, right: 0}}
                onClick={() => {
                    let new_path = location.pathname.split("/");
                    new_path.pop();

                    history.push(new_path.join("/"));
                }}
            />
            {loading ?
                <h4>Loading</h4>
                :
                user != null ?
                    <UserInformationView user={user}/>
                    :
                    <h4>No user information to show</h4>
            }
        </Stack>
    </Stack>
}

export default UserIdView;