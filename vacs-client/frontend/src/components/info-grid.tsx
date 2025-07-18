type InfoGridProps = {
    displayName: string
};

function InfoGrid(props: InfoGridProps) {
    return (
        <div className="grid grid-rows-2 w-full h-full" style={{ gridTemplateColumns: "25% 32.5% 42.5%" }}>
            <div className="info-grid-cell"></div>
            <div className="info-grid-cell"></div>
            <div className="info-grid-cell"></div>
            <div className="info-grid-cell">{props.displayName}</div>
            <div className="info-grid-cell"></div>
            <div className="info-grid-cell"></div>
        </div>
    );
}

export default InfoGrid;