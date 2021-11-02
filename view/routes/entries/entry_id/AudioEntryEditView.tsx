import { DefaultButton, IconButton, Stack, Text, Toggle } from "@fluentui/react";
import React, { useContext, useEffect, useRef, useState } from "react"
import { getUserMedia } from "../../../util/media";
import { AudioEntryState, AudioEntryUI, EntryIdViewContext, entry_id_view_actions } from "./reducer";

interface AudioEntryEditViewProps {
    audio_entries: AudioEntryState[]

    recording_in_progress: boolean
}

const AudioEntryEditView = ({audio_entries, recording_in_progress}: AudioEntryEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);
    let audio_ele_ref = useRef<HTMLAudioElement>(null);
    let media_ref = useRef<{
        stream: MediaStream,
        recorder: MediaRecorder,
        buffer: Blob[],
        blob: Blob
    }>({
        stream: null,
        recorder: null,
        buffer: [],
        blob: null
    });
    let [msg, setMsg] = useState(" ");
    let [recording_started, setRecordingStarted] = useState(false);
    let [recording_paused, setRecordingPaused] = useState(false);
    let [recording_finished, setRecordingFinished] = useState(false);

    const stopStreams = () => {
        if (media_ref.current.stream != null) {
            media_ref.current.stream.getTracks().forEach(track => {
                if (track.readyState === "live") {
                    track.stop();
                }
            });
        }
    }

    const createMediaRecorder = (): Promise<MediaRecorder> => {
        return getUserMedia({audio: true}).then((result) => {
            if (result == null) {
                setMsg("failed to get audio stream for recording");

                return null;
            } else {
                media_ref.current.stream = result;
                const media_recorder = new MediaRecorder(result, {mimeType: "audio/webm"});
                media_ref.current.recorder = media_recorder;

                media_recorder.addEventListener("dataavailable", (e) => {
                    if (e.data.size > 0) {
                        media_ref.current.buffer.push(e.data);
                    }
                });

                media_recorder.addEventListener("stop", (e) => {
                    setRecordingStarted(false);

                    if (media_ref.current.blob != null) {
                        media_ref.current.blob = new Blob(
                            [
                                media_ref.current.blob,
                                ...media_ref.current.buffer
                            ]
                        );
                    } else {
                        media_ref.current.blob = new Blob(media_ref.current.buffer);
                    }

                    media_ref.current.buffer = [];

                    if (audio_ele_ref.current != null) {
                        audio_ele_ref.current.src = URL.createObjectURL(
                            media_ref.current.blob
                        );
                    }

                    stopStreams();
                    setRecordingFinished(true);
                    dispatch(entry_id_view_actions.set_recording(false));
                });

                media_recorder.addEventListener("pause", (e) => {
                    setRecordingPaused(true);
                });

                media_recorder.addEventListener("start", (e) => {
                    setRecordingStarted(true);
                    dispatch(entry_id_view_actions.set_recording(true));
                });

                media_recorder.addEventListener("resume", (e) => {
                    setRecordingPaused(false);
                });

                media_recorder.addEventListener("error", (e) => {
                    console.log("media recorder error", e);
                });

                return media_recorder;
            }
        }).catch(err => {
            if (err.name === "AbortError") {
                setMsg("something caused an error. aborting");
            } else if (err.name === "NotAllowedError") {
                setMsg("the site is not allowed to access your microphone");
            } else if (err.name === "NotFoundError") {
                setMsg("no audio recording device was found for the system");
            } else {
                setMsg("unhandled error: " + err.name);
            }

            return null;
        })
    }

    const startRecording = () => {
        if (media_ref.current.recorder == null) {
            createMediaRecorder().then(recorder => {
                recorder?.start(1000);
            });
        } else {
            media_ref.current.recorder.start(1000);
        }
    }

    const stopRecording = () => {
        if (media_ref.current.recorder != null) {
            if (media_ref.current.recorder.state === "recording" ||
                media_ref.current.recorder.state === "paused"
            ) {
                media_ref.current.recorder.stop();
            }
        }
    }

    const pauseRecording = () => {
        if (media_ref.current.recorder != null) {
            if (media_ref.current.recorder.state === "recording") {
                media_ref.current.recorder.pause();
            }
        }
    }

    const resumeRecording = () => {
        if (media_ref.current.recorder != null) {
            if (media_ref.current.recorder.state === "paused") {
                media_ref.current.recorder.resume();
            }
        }
    }

    useEffect(() => {
        return () => stopStreams();
    }, []);

    return <Stack tokens={{childrenGap: 8}}>
        {audio_entries.map((v, index) => {
            return <Stack key={v.data.key ?? v.data.id} tokens={{childrenGap: 8}}>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    {!recording_finished ?
                        <>
                            <DefaultButton
                                text={recording_started ? "Stop" : "Start"}
                                disabled={recording_in_progress}
                                onClick={() => {
                                    if (recording_in_progress) {
                                        return;
                                    }

                                    if (recording_started) {
                                        stopRecording();
                                    } else {
                                        startRecording();
                                    }
                                }}
                            />
                            <DefaultButton
                                text={recording_paused ? "Resume" : "Pause"}
                                disabled={!recording_started}
                                onClick={() => {
                                    if (recording_paused) {
                                        resumeRecording();
                                    } else {
                                        pauseRecording();
                                    }
                                }}
                            />
                        </>
                        :
                        null
                    }
                    <audio ref={audio_ele_ref} controls/>
                </Stack>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Toggle label="Private" inlineLabel onText="Yes" offText="No" checked={v.data.private} onChange={(e, checked) => {
                        dispatch(entry_id_view_actions.update_audio_entry({index, private: checked}));
                    }}/>
                    <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                        dispatch(entry_id_view_actions.delete_audio_entry(index));
                    }}/>
                </Stack>
                <Text>{msg}</Text>
            </Stack>
        })}
    </Stack>
}

export default AudioEntryEditView;