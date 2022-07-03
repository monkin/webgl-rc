import React from "react";
import * as ReactDOM from "react-dom/client";
import { Canvas } from "./canvas";

(async () => {
    await Promise.resolve();

    const { create_context, draw_triangle } = await import("..");

    function GlExample() {
        return <Canvas init={create_context} paint={draw_triangle} />;
    }

    ReactDOM.createRoot(document.getElementById("app")!).render(<GlExample />);
})();
