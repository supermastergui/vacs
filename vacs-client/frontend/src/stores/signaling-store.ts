import {ClientInfo} from "../types/client-info.ts";
import {create} from "zustand/react";

type SignalingState = {
    connected: boolean;
    displayName: string;
    clients: ClientInfo[];
    setConnected: (connected: boolean) => void;
    setDisplayName: (displayName: string) => void;
    setClients: (clients: ClientInfo[]) => void;
}

export const useSignalingStore = create<SignalingState>()((set) => ({
    connected: false,
    displayName: "",
    clients: [],
    setConnected: (connected) => set({connected}),
    setDisplayName: (displayName) => set({displayName}),
    setClients: (clients) => set({clients}),
}));