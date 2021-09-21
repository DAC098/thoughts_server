import { DefaultButton, Stack, TextField } from "@fluentui/react"
import React, { useState } from "react"
import { useHistory, useLocation } from "react-router";
import useAppDispatch from "../../hooks/useAppDispatch"
import { actions } from "../../redux/slices/active_user"
import api from "../../apiv2"
import { urlFromLocation } from "../../util/url";

const Login = () => {
    const location = useLocation();
    const dispatch = useAppDispatch();

    let [username, setUsername] = useState("");
    let [password, setPassword] = useState("");

    let [sending, setSending] = useState(false);
    let [view_create, setViewCreate] = useState(false);
    let [username_error, setUsernameError] = useState("");
    let [password_error, setPasswordError] = useState("");

    const history = useHistory();

    const login = () => {
        if (sending) {
            return;
        }

        setSending(true);
        setUsernameError("");
        setPasswordError("");

        api.auth.login.post({post: {username,password}}).then((res) => {
            let user = res.body.data;
            let url = urlFromLocation(location);
            
            dispatch(actions.set_user(user));

            history.push(url.searchParams.get("jump_to") ?? "/entries");
        }).catch(err => {
            if (err.type === "UsernameNotFound") {
                setUsernameError(err.message);
            } else if (err.type === "InvalidPassword") {
                setPasswordError(err.message);
            }
        }).then(() => {setSending(false)});
    }

    return <Stack verticalAlign="center" horizontalAlign="center" style={{
        width: "100vw",
        height: "100vh"
    }}>
        <form
            onSubmit={e => {
                e.preventDefault();
                login();
            }}
        >
            <Stack tokens={{childrenGap: 8}}>
                <TextField 
                    label={view_create ? "Username" : "Username / Email"} 
                    required 
                    type="text" 
                    name="username"
                    value={username}
                    errorMessage={username_error}
                    onChange={(e,v) => setUsername(v)}
                />
                <TextField 
                    label="Password" 
                    required
                    type="password" 
                    name="password" 
                    canRevealPassword
                    value={password}
                    errorMessage={password_error}
                    onChange={(e,v) => setPassword(v)}
                />
                <DefaultButton 
                    primary 
                    text={view_create ? "Create" : "Login"}
                    type="submit" 
                    disabled={sending}
                />
            </Stack>
        </form>
    </Stack>
}

export default Login;