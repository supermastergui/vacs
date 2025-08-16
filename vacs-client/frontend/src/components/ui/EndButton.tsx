import Button from "./Button.tsx";
import {navigate} from "wouter/use-browser-location";
import {invokeStrict} from "../../error.ts";
import {useCallStore} from "../../stores/call-store.ts";
import {useAsyncDebounce} from "../../hooks/debounce-hook.ts";

function EndButton() {
    const callDisplay = useCallStore(state => state.callDisplay);
    const {endCall, dismissRejectedPeer} = useCallStore(state => state.actions);

    const endAnyCall = useAsyncDebounce(async () => {
        if (callDisplay?.type === "accepted" || callDisplay?.type === "outgoing") {
            try {
                await invokeStrict("signaling_end_call", {peerId: callDisplay.peerId});
                endCall();
            } catch {}
        } else if (callDisplay?.type === "rejected") {
            dismissRejectedPeer();
        }
    });

    const handleOnClick = async () => {
        navigate("/");

        void endAnyCall();
    };

    return (
        <Button color="cyan" className="text-xl w-44 px-10" onClick={handleOnClick}>END</Button>
    );
}

export default EndButton;