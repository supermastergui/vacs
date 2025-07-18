import Button, {ButtonProps} from "./button.tsx";
import {clsx} from "clsx";

type ActionButtonProps = {
    active?: boolean
} & ButtonProps;

function ActionButton(props: ActionButtonProps) {
    return (
        <Button className={clsx("bg-cyan-200 border-2 active:!border border-t-cyan-100 border-l-cyan-100 border-r-cyan-950 border-b-cyan-950 rounded-md active:!border-cyan-900", props.className)}>{props.children}</Button>
    );
}

export default ActionButton;