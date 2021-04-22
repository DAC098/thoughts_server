import { DefaultButton, PrimaryButton, TextField } from "@fluentui/react"
import React, { useState } from "react"
import { useHistory } from "react-router";
import { json } from "../../request";

const Login = () => {
    let [username, setUsername] = useState("");
    let [password, setPassword] = useState("");
    let [sending, setSending] = useState(false);
    let [username_error, setUsernameError] = useState(" ");
    let [password_error, setPasswordError] = useState(" ");

    let history = useHistory();

    return <form
        onSubmit={e => {
            e.preventDefault();

            if (sending) {
                return;
            }

            setSending(true);
            setUsernameError(" ");
            setPasswordError(" ");

            json.post("/auth/login", {username,password})
                .then(({body,response,status}) => {
                    history.push("/entries");
                })
                .catch(err => {
                    console.log(err, err.type, err.status);
                    if (err.type === "UsernameNotFound") {
                        setUsernameError(err.message);
                    } else if (err.type === "InvalidPassword") {
                        setPasswordError(err.message);
                    }
                })
                .then(() => {setSending(false)});
        }}
    >
        <TextField 
            label="Username / Email" 
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
            text="Login" 
            type="submit" 
            disabled={sending}
        />
    </form>
}

export default Login;