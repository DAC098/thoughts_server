import { DefaultButton, Icon, Label, Persona, PersonaSize, Stack, TextField } from "@fluentui/react"
import React, { useEffect, useState } from "react"
import useAppDispatch from "../hooks/useAppDispatch"
import useAppSelector from "../hooks/useAppSelector"
import { json } from "../request"
import { actions as active_user_actions } from "../redux/slices/active_user"
import IndentSection from "../components/IndentSection"

const PasswordSection = () => {
    let [current, setCurrent] = useState("");
    let [new_password, setNewPassword] = useState("");
    let [confirm_password, setConfirmPassword] = useState("");
    let [confirm_error, setConfirmError] = useState("");
    let [sending, setSending] = useState(false);

    const sendUpdate = () => {
        if (sending) {
            return;
        }

        if (new_password !== confirm_password) {
            setConfirmError("your new password does not match. your password must match before saving.");
            return;
        }

        setSending(true);

        json.post("/auth/change", {
            current_password: current,
            new_password
        }).then(() => {
            setCurrent("");
            setNewPassword(""),
            setConfirmPassword("");
        }).catch(console.error).then(() => {
            setSending(false);
        });
    }

    return <IndentSection content="Password" stack={{styles: {"root": {width: 250}}}}>
        <TextField
            label="Current Password"
            type="password"
            name="password"
            canRevealPassword
            onChange={(e,v) => setCurrent(v)}
        />
        <TextField
            label="New Password"
            type="password"
            name="new_password"
            canRevealPassword
            onChange={(e,v) => setNewPassword(v)}
        />
        <TextField 
            label="Confirm New Password" 
            type="password"
            name="confirm_password"
            canRevealPassword
            errorMessage={confirm_error}
            onChange={(e,v) => {
                setConfirmError("");
                setConfirmPassword(v);
            }}
        />
        <Stack.Item>
            <DefaultButton
                text="Save"
                disabled={
                    current.length === 0 || 
                    new_password.length === 0 ||
                    confirm_password.length === 0 ||
                    sending
                }
                onClick={sendUpdate}
            />
        </Stack.Item>
    </IndentSection>
}

const UserInformation = () => {
    const active_user_state = useAppSelector(state => state.active_user);
    const appDispatch = useAppDispatch();

    let [full_name, setFullName] = useState(active_user_state.user.full_name);
    let [username, setUsername] = useState(active_user_state.user.username);
    let [email, setEmail] = useState(active_user_state.user.email);
    let [sending, setSending] = useState(false);

    const sendUpdate = () => {
        if (sending) {
            return;
        }

        setSending(true);

        json.put<any>("/account", {
            full_name, username, email
        }).then(({body}) => {
            appDispatch(active_user_actions.update_info({
                username: body.data.username, 
                full_name: body.data.full_name, 
                email: body.data.email,
                email_verified: body.data.email_verified
            }));
        }).catch(console.error).then(() => {
            setSending(false);
        })
    }

    useEffect(() => {
        setUsername(active_user_state.user.username);
        setFullName(active_user_state.user.full_name);
        setEmail(active_user_state.user.email);
    }, [
        active_user_state.user.username, 
        active_user_state.user.full_name, 
        active_user_state.user.email
    ]);

    return <>
        <Stack horizontal>
            <Persona size={PersonaSize.size120}/>
            <Stack tokens={{childrenGap: 8}}>
                <TextField placeholder="Full Name" value={full_name} onChange={(e,v) => setFullName(v)}/>
                <TextField placeholder="Username" value={username} onChange={(e,v) => setUsername(v)}/>
            </Stack>
        </Stack>
        <IndentSection content="Personal Information">
            <Stack styles={{root: {width: 250}}}>
                <Label>
                    Email
                    {active_user_state.user.email_verified ?
                        <Icon styles={{root: {marginLeft: 4}}} iconName="VerifiedBrandSolid"/>
                        :
                        null
                    } 
                </Label>
                <TextField value={email ?? ""} onChange={(e,v) => setEmail(v)}/>
            </Stack>
            <Stack.Item>
                <DefaultButton 
                    text="Save"
                    disabled={sending}
                    onClick={sendUpdate}
                />
            </Stack.Item>
        </IndentSection>
    </>
}

const AccountView = () => {
    return <Stack 
        styles={{
            "root": {
                wdith: "100%", height: "100%",
                position: "relative"
            }
        }} 
        tokens={{childrenGap: 8, padding: 8}}
    >
        <UserInformation/>
        <IndentSection content="Authentication">
            <PasswordSection/>
        </IndentSection>
    </Stack>
}

export default AccountView;