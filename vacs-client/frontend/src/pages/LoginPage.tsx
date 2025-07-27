import {invokeSafe} from "../error.ts";
import {useAsyncDebounce} from "../hooks/debounce-hook.ts";

function LoginPage() {
    const handleLoginClick = useAsyncDebounce(async () => {
        await invokeSafe("open_auth_url");
    });

    return (
        <div className="h-full w-full flex justify-center items-center p-4">
            <button
                className="px-6 py-2  border border-[rgba(0,0,0,.35)] text-amber-50 rounded cursor-pointer text-lg"
                style={{background: "linear-gradient(to bottom left, #2483C5 0%, #29B473 100%) border-box"}}
                onClick={handleLoginClick}
            >
                Login via VATSIM
            </button>
        </div>
    );
}

export default LoginPage;