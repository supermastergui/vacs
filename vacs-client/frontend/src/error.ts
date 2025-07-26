import {invoke, InvokeArgs} from "@tauri-apps/api/core";

export interface FrontendError {
    title: string;
    message: string;
    timeout_ms?: number;
}

export const safeInvoke = async <T>(cmd: string, args?: InvokeArgs): Promise<T> => {
    try {
        return await invoke<T>(cmd, args);
    } catch (err) {
        throw parseFrontendError(err);
    }
}

const parseFrontendError = (err: unknown): FrontendError => {
    if (typeof err === "object" && err !== null && "title" in err && "message" in err) {
        return {
            title: err.title as string,
            message: err.message as string,
            timeout_ms: (err as any).timeout_ms ?? undefined,
        }
    }

    if (typeof err === "string") {
        try {
            return JSON.parse(err) as FrontendError;
        } catch {
            // not a JSON serialized error, fallthrough
        }

        return {
            title: "Error",
            message: err,
        };
    }

    return {
        title: "Unexpected error",
        message: "An unknown error occurred",
    }
}