import Button, {ButtonProps} from "./button.tsx";
import {clsx} from "clsx";

type NavigationButtonProps = ButtonProps;

function NavigationButton(props: NavigationButtonProps) {
    return (
        <Button
            className={clsx(
                "bg-gray-300 border-2 active:!border border-t-gray-100 border-l-gray-100 border-r-gray-700 " +
                "border-b-gray-700 active:!border-gray-700 shadow-[0_0_0_1px_#364153] active:shadow-none text-lg",
                props.className)}
        >
            {props.children}
        </Button>
    );
}

export default NavigationButton;