import React from "react";
import styled from "styled-components";

export interface CanvasProps<C> {
    init: (canvas: HTMLCanvasElement) => C;
    paint: (context: C, width: number, height: number) => void;
}

const SizedCanvas = styled.canvas`
  width: 100%;
  height: 100%;
  display: block;
  margin: 0;
  padding: 0;
  border-width: 0;
`;

export function Canvas<C extends { free: () => void; }>({ init, paint }: CanvasProps<C>) {
    const node = React.useRef<HTMLCanvasElement>(null);
    React.useEffect(() => {
        const canvas = node.current!;
        let context = init(canvas);

        let size = [0, 0];

        function resize() {
            if (!disposed) {
                size = [canvas.clientWidth * devicePixelRatio, canvas.clientHeight * devicePixelRatio];
                canvas.setAttribute("width", size[0].toFixed(0));
                canvas.setAttribute("height", size[1].toFixed(0));
            }
        }

        function requestResize() {
            resize();
            setTimeout(resize, 200);
        }

        let disposed = false;
        function requestRepaint() {
            requestAnimationFrame(() => {
                if (!disposed) {
                    paint(context, size[0], size[1]);
                    requestRepaint();
                }
            });
        }

        requestRepaint();
        requestResize();

        window.addEventListener("resize", requestResize);
        window.addEventListener("orientationchange", requestResize);

        return () => {
            disposed = true;
            window.removeEventListener("resize", requestResize);
            window.removeEventListener("orientationchange", requestResize);
            context.free();
        };
    }, [init, paint]);
    return <SizedCanvas ref={node}/>;
}