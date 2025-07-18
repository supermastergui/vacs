import ActionButton from "./ui/action-button.tsx";
import {Link} from "wouter";
import wrenchAndDriver from "../assets/wrench-and-driver.svg";

function TopButtonRow() {
    return (
        <div className="h-22 w-full flex flex-row gap-2 justify-between p-2 [&>button]:px-3">
            <ActionButton>PRIO</ActionButton>
            <ActionButton>HOLD</ActionButton>
            <ActionButton>PICKUP</ActionButton>
            <ActionButton className="whitespace-pre-wrap">
                SUITE<br/>PICKUP
            </ActionButton>
            <ActionButton>TRANS</ActionButton>
            <ActionButton>DIV</ActionButton>
            <ActionButton>
                PLAY<br/>BACK
            </ActionButton>
            <ActionButton className="text-slate-400">
                PLC<br/>LSP<br/>on/off
            </ActionButton>
            <ActionButton>SPLIT</ActionButton>
            <Link to="/settings">
                <ActionButton className="h-full flex justify-center items-center">
                    <img src={wrenchAndDriver} alt="Settings" className="h-12 w-12" />
                </ActionButton>
            </Link>
            <ActionButton className="min-w-24"></ActionButton>
        </div>
    );
}

export default TopButtonRow;