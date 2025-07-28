import {listen, UnlistenFn} from "@tauri-apps/api/event";
import {useSignalingStore} from "../stores/signaling-store.ts";
import {ClientInfo} from "../types/client-info.ts";

export function setupSignalingListeners() {
    const { setConnected, setDisplayName, setClients } = useSignalingStore.getState();

    const unlistenFns: (Promise<UnlistenFn>)[] = [];

    const init = () => {
        const unlisten1 = listen<string>("signaling:connected", (event) => {
            setConnected(true);
            setDisplayName(event.payload);
        });

        const unlisten2 = listen("signaling:disconnected", () => {
            setConnected(false);
            setDisplayName("");
        });

        const unlisten3 = listen<ClientInfo[]>("signaling:client-list", (event) => {
            setClients(event.payload);
        });

        unlistenFns.push(unlisten1, unlisten2, unlisten3);
    };

    init();

    return () => {
        unlistenFns.forEach(fn => fn.then(f => f()));
    }
}