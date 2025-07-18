import {ComponentChildren} from "preact";
import {clsx} from "clsx";

export type ButtonProps = {
    className?: string;
    children?: ComponentChildren;
    onClick?: (event: MouseEvent) => void;
}

function Button(props: ButtonProps) {
    return (
        <button
            className={clsx("leading-5 w-24 font-bold cursor-pointer",
                props.className)}
            onClick={props.onClick}
        >
            {props.children}
        </button>
    );
}

export default Button;