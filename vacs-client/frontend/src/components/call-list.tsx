import NavigationButton from "./ui/navigation-button.tsx";
import {useState} from "preact/hooks";

function CallList() {
    const [contacts, setContacts] = useState<string[]>(["LOVV_CTR", "LOVV_CTR",
        "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR", "LOVV_CTR"
    ]);

    return (
        <div className="flex flex-col flex-wrap h-full overflow-hidden p-2 gap-3 relative">
            {contacts.map(contact =>
                <NavigationButton className="w-24 h-18 rounded !text-base !leading-4.5 !font-semibold">{contact}</NavigationButton>
            )}
            {/*<div className="w-5 h-5 bg-red-500 absolute top-[50%]"></div> 320-340<br/>E2<br/>EC*/}
        </div>
    );
}

export default CallList;