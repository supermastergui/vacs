import "./App.css";
import Clock from "./components/clock.tsx";
import InfoGrid from "./components/info-grid.tsx";
import ActionButton from "./components/ui/action-button.tsx";
import NavigationButton from "./components/ui/navigation-button.tsx";
import CallList from "./components/call-list.tsx";
import MainArea from "./components/main-area.tsx";
import TopButtonRow from "./components/top-button-row.tsx";

function App() {
    return (
        <div className="h-screen flex flex-col">
            <div className="w-full h-12 bg-gray-300 flex flex-row border-gray-700 border-b">
                <Clock />
                <InfoGrid displayName="N36 PLC" />
            </div>
            <div className="w-full h-[calc(100%-3rem)] flex flex-col">
                {/* Top Button Row */}
                <TopButtonRow />
                <div className="flex flex-row w-full h-[calc(100%-11rem)] pl-1">
                    {/* Main Area */}
                    <div className="h-full w-[calc(100%-7rem)] overflow-hidden bg-gray-300 border-l-1 border-t-1 border-r-2 border-b-2 border-gray-700 rounded-sm flex flex-row">
                        <CallList />
                    </div>
                    {/* Right Button Row */}
                    <div className="w-28 h-full px-2 pb-6">
                        <ActionButton className="h-18"></ActionButton>
                        <div className="h-[calc(100%-4.2rem)] flex flex-col justify-end gap-3 pt-3 pr-[1px]">
                            <div className="w-full border rounded h-20"></div>
                            <div className="w-full border rounded h-20"></div>
                            <div className="w-full border rounded h-20"></div>
                            <div className="w-full border rounded h-20"></div>
                            <div className="w-full border rounded h-20"></div>
                            <div className="w-full border rounded h-20"></div>
                        </div>
                    </div>
                </div>
                {/* Bottom Button Row */}
                <div className="h-22 w-full p-2 pl-4 flex flex-row justify-between gap-20">
                    <div className="h-full flex flex-row gap-3 shrink">
                        <NavigationButton className="text-xl w-46 font-semibold rounded-md">Radio</NavigationButton>
                        <ActionButton className="text-xl font-semibold">CPL</ActionButton>
                        <ActionButton className="text-xl w-46 font-semibold text-slate-400">
                            RADIO<br/>PRIO
                        </ActionButton>
                        <NavigationButton className="text-xl w-46 font-semibold rounded-md">Phone</NavigationButton>
                    </div>
                    <ActionButton className="text-xl w-44 font-semibold px-10">END</ActionButton>
                </div>
            </div>
        </div>
    );
}

export default App;
