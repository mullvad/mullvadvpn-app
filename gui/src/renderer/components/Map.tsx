import { mat4 } from 'gl-matrix';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import styled from 'styled-components';

import GLMap, { Coordinate } from '../lib/map/3dmap';

// The angle in degrees that the camera sees in
const angleOfView = 70;

export enum MarkerStyle {
  secure,
  unsecure,
}

const StyledCanvas = styled.canvas({
  position: 'absolute',
  width: '100%',
  height: '100%',
});

interface MapParams {
  location: Coordinate;
  connectionState: boolean;
}

interface MapProps {
  location: [number, number];
  markerStyle: MarkerStyle;
}

export default function Map(props: MapProps) {
  // TODO: Change order of long/lat in Coordinate
  const coordinate = useMemo(() => new Coordinate(props.location[1], props.location[0]), [
    ...props.location,
  ]);

  // Callback that should be passed to requestAnimationFrame. This is initialized after the canvas
  // has been rendered.
  const frameCallback = useRef<(now: number) => void>();
  // When location or connection state changes it's stored here until passed to 3dmap
  const newParams = useRef<MapParams>();
  // This is set to true when rendering should be paused
  const pause = useRef<boolean>(false);

  // Called when the canvas has been rendered
  const canvasRef = useCallback((canvas: HTMLCanvasElement | null) => {
    if (!canvas) {
      return;
    }

    const innerFrameCallback = getAnimationFramCallback(
      canvas,
      coordinate,
      props.markerStyle === MarkerStyle.secure,
      () => (pause.current = true),
    );

    frameCallback.current = (now: number) => {
      innerFrameCallback(now, newParams.current);
      // Clear new params to avoid setting them multiple times
      newParams.current = undefined;

      // Stops recursively requesting to be called the next frame when it should be paused
      if (!pause.current) {
        requestAnimationFrame(frameCallback.current!);
      }
    };

    requestAnimationFrame(frameCallback.current);
  }, []);

  // Set new params when the location or connection state has changed, and unpause if paused
  useEffect(() => {
    newParams.current = {
      location: coordinate,
      connectionState: props.markerStyle === MarkerStyle.secure,
    };

    if (pause.current) {
      pause.current = false;
      if (frameCallback.current) {
        requestAnimationFrame(frameCallback.current);
      }
    }
  }, [coordinate, props.markerStyle]);

  // TODO: Properly detect height
  return <StyledCanvas ref={canvasRef} id="glcanvas" width={window.innerWidth} height="493" />;
}

type AnimationFrameCallback = (now: number, newParams?: MapParams) => void;

function getAnimationFramCallback(
  canvas: HTMLCanvasElement,
  startingCoordinate: Coordinate,
  connectionState: boolean,
  animationEndListener: () => void,
): AnimationFrameCallback {
  const gl = canvas.getContext('webgl2', { antialias: true })!;
  setGlOptions(gl);

  const projectionMatrix = getProjectionMatrix(gl);

  const map = new GLMap(gl, startingCoordinate, connectionState, animationEndListener);

  const drawScene = (now: number) => {
    gl.clearColor(0.0, 0.0, 0.0, 1.0); // Clear to black, fully opaque
    gl.clearDepth(1.0); // Clear everything
    gl.enable(gl.DEPTH_TEST); // Enable depth testing
    gl.depthFunc(gl.LEQUAL); // Near things obscure far things

    // Clear the canvas before we start drawing on it.
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    map.draw(projectionMatrix, now);
  };

  const frameCallback = (now: number, newParams?: MapParams) => {
    now *= 0.001; // convert to seconds

    if (newParams) {
      map.setLocation(newParams.location, newParams.connectionState, now);
    }

    drawScene(now);
  };

  return frameCallback;
}

function setGlOptions(gl: WebGL2RenderingContext) {
  // Hide triangles not facing the camera
  gl.enable(gl.CULL_FACE);
  gl.cullFace(gl.BACK);

  // Enable transparency (alpha < 1.0)
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
}

function getProjectionMatrix(gl: WebGL2RenderingContext): mat4 {
  // Enables using gl.UNSIGNED_INT for indexes. Allows 32 bit integer
  // indexes. Needed to have more than 2^16 vertices in one buffer.
  // Not needed on WebGL2 canvases where it's enabled by default
  // const ext = gl.getExtension('OES_element_index_uint');

  // Create a perspective matrix, a special matrix that is
  // used to simulate the distortion of perspective in a camera.
  const fieldOfView = (angleOfView / 180) * Math.PI; // in radians
  // @ts-ignore
  const aspect = gl.canvas.clientWidth / gl.canvas.clientHeight;
  const zNear = 0.1;
  const zFar = 10;
  const projectionMatrix = mat4.create();
  mat4.perspective(projectionMatrix, fieldOfView, aspect, zNear, zFar);

  return projectionMatrix;
}
