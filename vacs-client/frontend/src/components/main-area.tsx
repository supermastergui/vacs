import NavigationButton from "./ui/navigation-button.tsx";

function MainArea() {
    return (
        <>
            <div className="w-full h-full border-r-4 border-r-gray-500 flex flex-row">
                <div className="h-full p-2 flex flex-col justify-between">
                    <NavigationButton className="w-30 h-30">KAR_MUN</NavigationButton>
                    <NavigationButton className="w-30 h-30">KAR_MUN</NavigationButton>
                    <NavigationButton className="w-30 h-30">ZUR</NavigationButton>
                </div>
                <div className="w-full flex flex-col">
                    <div className="w-full flex flex-row justify-between p-2">
                        <NavigationButton className="w-30 h-30">FIC</NavigationButton>
                        <NavigationButton className="w-30 h-30">Cont</NavigationButton>
                        <NavigationButton className="w-30 h-30">FDU</NavigationButton>
                    </div>
                    <div className="h-full w-full flex flex-col">
                        <div className="py-1 h-full w-full flex flex-row gap-3 items-end justify-center">
                            <NavigationButton className="w-50 h-50">
                                <span className="block">B</span>
                                <span className="block">LOWS</span>
                            </NavigationButton>
                            <NavigationButton className="w-50 h-50">
                                <span className="block">N</span>
                                <span className="block">LOWL</span>
                            </NavigationButton>
                            <NavigationButton className="w-50 h-50">
                                <span className="block">E</span>
                                <span className="block">APP</span>
                            </NavigationButton>
                        </div>
                        <div className="py-1 h-full w-full flex flex-row gap-3 items-start justify-center">
                            <NavigationButton className="w-50 h-50">
                                <span className="block">W</span>
                                <span className="block">WI_WK</span>
                            </NavigationButton>
                            <NavigationButton className="w-50 h-50">
                                <span className="block">S</span>
                                <span className="block">LOWG</span>
                            </NavigationButton>
                        </div>
                    </div>
                    <div className="w-full flex flex-row justify-between px-20 py-2">
                        <NavigationButton className="w-30 h-30">PAD</NavigationButton>
                        <NavigationButton className="w-30 h-30">LJU</NavigationButton>
                    </div>
                </div>
                <div className="h-full p-2 pr-4 flex flex-col justify-between">
                    <NavigationButton className="w-30 h-30">PRA</NavigationButton>
                    <NavigationButton className="w-30 h-30">BRA</NavigationButton>
                    <NavigationButton className="w-30 h-30">BUD</NavigationButton>
                    <NavigationButton className="w-30 h-30">ZAG</NavigationButton>
                </div>
            </div>
            <div className="h-full p-2 pl-4 flex flex-col justify-between">
                <NavigationButton className="w-30 h-30">MIL</NavigationButton>
                <NavigationButton className="w-30 h-30">FMP</NavigationButton>
                <NavigationButton className="w-30 h-30">SUP</NavigationButton>
                <NavigationButton className="w-30 h-30">CWP</NavigationButton>
            </div>
        </>
    );
}

export default MainArea;