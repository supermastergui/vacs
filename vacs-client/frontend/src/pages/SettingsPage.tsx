import Button from "../components/ui/Button.tsx";
import {navigate} from "wouter/use-browser-location";
import {useAuthStore} from "../stores/auth-store.ts";
import {invokeSafe, invokeStrict} from "../error.ts";
import {useAsyncDebounce} from "../hooks/debounce-hook.ts";
import {useSignalingStore} from "../stores/signaling-store.ts";
import DeviceSelector from "../components/DeviceSelector.tsx";
import VolumeSettings from "../components/VolumeSettings.tsx";
import AudioHostSelector from "../components/AudioHostSelector.tsx";
import {useEffect, useState} from "preact/hooks";
import {getCurrentWindow} from "@tauri-apps/api/window";

function SettingsPage() {
    return (
        <div className="h-full w-full bg-blue-700 border-t-0 px-2 pb-2 flex flex-col overflow-auto">
            <p className="w-full text-white bg-blue-700 font-semibold text-center">Settings</p>
            <div className="w-full grow rounded-b-sm bg-[#B5BBC6] flex flex-col">
                <div className="w-full grow border-b-2 border-zinc-200 flex flex-row">
                    <VolumeSettings/>
                    <div className="h-full grow flex flex-col">
                        <p className="w-full text-center border-b-2 border-zinc-200 uppercase font-semibold">Devices</p>
                        <div className="w-full grow px-3 py-1.5 flex flex-col">
                            <AudioHostSelector/>
                            <DeviceSelector deviceType="Output"/>
                            <DeviceSelector deviceType="Input"/>
                        </div>
                    </div>
                </div>
                <div
                    className="h-20 w-full flex flex-row gap-2 justify-between p-2 [&>button]:px-1 [&>button]:shrink-0">
                    <AlwaysOnTopButton/>
                    <DisconnectLogoutButtons/>
                </div>
            </div>
        </div>
    );
}

function DisconnectLogoutButtons() {
    const connected = useSignalingStore(state => state.connected);
    const isAuthenticated = useAuthStore(state => state.status === "authenticated");

    const handleLogoutClick = useAsyncDebounce(async () => {
        try {
            await invokeStrict("auth_logout");
            navigate("/");
        } catch {
        }
    });

    const handleDisconnectClick = useAsyncDebounce(async () => {
        try {
            await invokeStrict("signaling_disconnect");
            navigate("/");
        } catch {
        }
    });

    return (
        <div className="h-full flex flex-row gap-2">
            <Button color="salmon" className="w-auto px-3 rounded" disabled={!connected}
                    onClick={handleDisconnectClick}>Disconnect</Button>
            <Button color="salmon" className="rounded" disabled={!isAuthenticated}
                    onClick={handleLogoutClick}>Logout</Button>
        </div>
    );
}

function AlwaysOnTopButton() {
    const [alwaysOnTop, setAlwaysOnTop] = useState<boolean>(false);

    const toggleAlwaysOnTop = useAsyncDebounce(async () => {
        const isAlwaysOnTop = await invokeSafe<boolean>("app_set_always_on_top", {alwaysOnTop: !alwaysOnTop});
        setAlwaysOnTop(alwaysOnTop => isAlwaysOnTop ?? alwaysOnTop);
    });

    useEffect(() => {
        const fetchIsAlwaysOnTop = async () => {
            const isAlwaysOnTop = await getCurrentWindow().isAlwaysOnTop();
            setAlwaysOnTop(alwaysOnTop => isAlwaysOnTop ?? alwaysOnTop);
        };

        void fetchIsAlwaysOnTop();
    }, []);

    return (
        <Button
            color={alwaysOnTop ? "blue" : "cyan"}
            className="rounded"
            onClick={toggleAlwaysOnTop}
        >
            <p>Always<br/>on Top</p>
        </Button>
    );
}

export default SettingsPage;