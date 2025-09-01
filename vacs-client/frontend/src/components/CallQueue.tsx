import Button from "./ui/Button.tsx";
import {useCallStore} from "../stores/call-store.ts";
import {invokeStrict} from "../error.ts";

function CallQueue() {
    const blink = useCallStore(state => state.blink);
    const callDisplay = useCallStore(state => state.callDisplay);
    const incomingCalls = useCallStore(state => state.incomingCalls);
    const {acceptCall, endCall, dismissRejectedPeer, dismissErrorPeer, removePeer} = useCallStore(state => state.actions);

    const handleCallDisplayClick = async (peerId: string) => {
        if (callDisplay?.type === "accepted" || callDisplay?.type === "outgoing") {
            try {
                await invokeStrict("signaling_end_call", {peerId: peerId});
                endCall();
            } catch {}
        } else if (callDisplay?.type === "rejected") {
            dismissRejectedPeer();
        } else if (callDisplay?.type === "error") {
            dismissErrorPeer();
        }
    };

    const handleAnswerKeyClick = async (peerId: string) => {
        // Can't call someone if you are currently in an active or outgoing/rejected call
        if (callDisplay !== undefined) return;

        try {
            acceptCall(peerId);
            await invokeStrict("signaling_accept_call", {peerId: peerId});
        } catch {
            removePeer(peerId);
        }
    }

    const cdColor = callDisplay?.type === "accepted" ? "green" : callDisplay?.type === "rejected" && blink ? "green" : callDisplay?.type === "error" && blink ? "red" : "gray";

    return (
        <div className="flex flex-col-reverse gap-2.5 pt-3 pr-[1px] overflow-y-auto" style={{scrollbarWidth: "none"}}>
            {/*Call Display*/}
            {callDisplay !== undefined ? (
                <Button color={cdColor}
                        highlight={callDisplay.type === "outgoing" || callDisplay.type === "rejected" ? "green" : undefined}
                        softDisabled={true}
                        onClick={() => handleCallDisplayClick(callDisplay.peerId)}
                        className={"min-h-16 text-sm"}>{callDisplay.peerId}</Button>
            ) : (
                <div className="w-full border rounded-md min-h-16"></div>
            )}

            {/*Answer Keys*/}
            {incomingCalls.map(peerId => (
                <Button color={blink ? "green" : "gray"} className={"min-h-16 text-sm"}
                        onClick={() => handleAnswerKeyClick(peerId)}>{peerId}</Button>
            ))}
            {Array.from(Array(Math.max(5 - incomingCalls.length, 0))).map(() => <div
                className="w-full border rounded-md min-h-16"></div>)}
        </div>
    );
}

export default CallQueue;