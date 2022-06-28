import React from "react";
import ReactDOM from "react-dom";
import { Canvas } from "./canvas";

(async () => {
    await Promise.resolve();

    const { create_context, draw_triangle } = await import("..");

    function GlExample() {
        return <Canvas init={create_context} paint={draw_triangle} />;
    }

    ReactDOM.render(<GlExample />, document.getElementById("app"));
})();
