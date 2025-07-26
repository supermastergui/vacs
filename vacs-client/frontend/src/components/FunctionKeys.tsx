import {Link, useLocation} from "wouter";
import Button from "./ui/Button.tsx";
import wrenchAndDriver from "../assets/wrench-and-driver.svg";
import mission from "../assets/mission.svg";

function FunctionKeys() {
    const [location] = useLocation();

    return (
        <div className="h-20 w-full flex flex-row gap-2 justify-between p-2 [&>button]:px-1 [&>button]:shrink-0">
            <Button color="cyan">PRIO</Button>
            <Button color="cyan">HOLD</Button>
            <Button color="cyan">PICKUP</Button>
            <Button color="cyan">
                <p>SUITE<br/>PICKUP</p>
            </Button>
            <Button color="cyan">TRANS</Button>
            <Button color="cyan">DIV</Button>
            <Button color="cyan">
                <p>PLAY<br/>BACK</p>
            </Button>
            <Button color="cyan" className="text-slate-400" disabled={true}>
                <p>PLC<br/>LSP<br/>on/off</p>
            </Button>
            <Button color="cyan">SPLIT</Button>
            <Link to={location === "/settings" ? "/" : "/settings"} draggable={false}>
                <Button color={location === "/settings" ? "blue" : "cyan"} className="h-full flex justify-center items-center">
                    <img src={wrenchAndDriver} alt="Settings" className="h-12 w-12" draggable={false} />
                </Button>
            </Link>
            <Button color="cyan" className="min-w-20 flex justify-center items-center">
                <img src={mission} alt="Settings" className="h-14 w-14" draggable={false} />
            </Button>
        </div>
    );
}

export default FunctionKeys;