import { DefaultButton, Stack, TextField } from "@fluentui/react"
import React, { useState } from "react"
import { useHistory } from "react-router";
import { useAppDispatch } from "../../hooks/useApp";
import { json } from "../../request";
import { actions } from "../../redux/slices/active_user"
import api from "../../api"

const Login = () => {
    const dispatch = useAppDispatch();

    let [username, setUsername] = useState("");
    let [password, setPassword] = useState("");
    let [confirm_password, setConfirmPassword] = useState("");
    let [email, setEmail] = useState("");

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

        api.login.post({username,password}).then((user) => {
            dispatch(actions.set_user(user));
            history.push("/entries");
        }).catch(err => {
            if (err.type === "UsernameNotFound") {
                setUsernameError(err.message);
            } else if (err.type === "InvalidPassword") {
                setPasswordError(err.message);
            }
        }).then(() => {setSending(false)});
    }

    const createLogin = () => {
        if (sending) {
            return;
        }

        if (password !== confirm_password) {
            setPasswordError("confirmation password is not the same");
            return;
        }

        setSending(true);
        setUsernameError("");
        setPasswordError("");

        json.post("/auth/create", {username,password,email}).then(() => {
            history.push("/entries")
        }).catch(err => {
            if (err.type === "UsernameExists") {
                setUsernameError(err.message);
            }
        }).then(() => setSending(false));
    }

    return <Stack verticalAlign="center" horizontalAlign="center" style={{
        width: "100vw",
        height: "100vh"
    }}>
        <form
            onSubmit={e => {
                e.preventDefault();

                if (view_create)
                    createLogin();
                else
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
                {view_create ?
                    <>
                        <TextField
                            label="Confirm Password"
                            required
                            type="password"
                            name="confirm_password"
                            canRevealPassword
                            value={confirm_password}
                            onChange={(e,v) => setConfirmPassword(v)}
                        />
                        <TextField
                            label="Email"
                            required
                            type="email"
                            name="email"
                            value={email}
                            onChange={(e,v) => setEmail(v)}
                        />
                    </>
                    :
                    null
                }
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Stack.Item>
                        <DefaultButton 
                            primary 
                            text={view_create ? "Create" : "Login"}
                            type="submit" 
                            disabled={sending}
                        />
                    </Stack.Item>
                    <Stack.Item>
                        <DefaultButton
                            text={view_create ? "Cancel" : "Create"}
                            disabled={sending}
                            onClick={() => setViewCreate(!view_create)}
                        />
                    </Stack.Item>
                </Stack>
            </Stack>
        </form>
    </Stack>
}

export default Login;