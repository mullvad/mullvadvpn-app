import { mat4 } from 'gl-matrix';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';

import GLMap, { ConnectionState, Coordinate } from '../lib/map/3dmap';
import { useCombinedRefs } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';

// The angle in degrees that the camera sees in
const angleOfView = 70;

const StyledCanvas = styled.canvas({
  position: 'absolute',
  width: '100%',
  height: '100%',
});

interface MapParams {
  location: Coordinate;
  connectionState: ConnectionState;
}

export default function Map() {
  const connection = useSelector((state) => state.connection);

  const location = useMemo<Coordinate>(() => {
    const { latitude, longitude } = connection;
    return typeof latitude === 'number' && typeof longitude === 'number'
      ? new Coordinate(latitude, longitude)
      : new Coordinate(0, 0);
  }, [connection.latitude, connection.longitude]);

  const connectionState = useMemo<ConnectionState>(() => {
    switch (connection.status.state) {
      case 'connecting':
      case 'connected':
        return ConnectionState.secure;
      case 'error':
        return !connection.status.details.blockingError
          ? ConnectionState.secure
          : ConnectionState.unsecure;
      case 'disconnected':
        return ConnectionState.unsecure;
      case 'disconnecting':
        switch (connection.status.details) {
          case 'block':
          case 'reconnect':
            return ConnectionState.secure;
          case 'nothing':
            return ConnectionState.unsecure;
        }
    }
  }, [connection.status]);

  const mapParams = useMemo<MapParams>(
    () => ({
      location,
      connectionState,
    }),
    [location, connectionState],
  );

  return <MapInner mapParams={mapParams} />;
}

interface MapInnerProps {
  mapParams: MapParams;
}

function MapInner(props: MapInnerProps) {
  // Callback that should be passed to requestAnimationFrame. This is initialized after the canvas
  // has been rendered.
  const frameCallback = useRef<(now: number) => void>();
  // When location or connection state changes it's stored here until passed to 3dmap
  const newParams = useRef<MapParams>();
  // This is set to true when rendering should be paused
  const pause = useRef<boolean>(false);

  const canvasRef = useRef<HTMLCanvasElement>();
  const [canvasWidth, setCanvasWidth] = useState(window.innerWidth);
  const [canvasHeight, setCanvasHeight] = useState(493);

  const updateCanvasSize = useCallback((canvas: HTMLCanvasElement) => {
    const canvasRect = canvas.getBoundingClientRect();
    canvas.width = canvasRect.width;
    canvas.height = canvasRect.height;
    setCanvasWidth(canvasRect.width);
    setCanvasHeight(canvasRect.height);
  }, []);

  // Called when the canvas has been rendered
  const canvasCallback = useCallback(
    (canvas: HTMLCanvasElement | null) => {
      if (!canvas) {
        return;
      }

      updateCanvasSize(canvas);

      const innerFrameCallback = getAnimationFramCallback(
        canvas,
        props.mapParams.location,
        props.mapParams.connectionState,
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
    },
    [updateCanvasSize],
  );

  const combinedCanvasRef = useCombinedRefs(canvasRef, canvasCallback);

  // Set new params when the location or connection state has changed, and unpause if paused
  useEffect(() => {
    newParams.current = {
      location: props.mapParams.location,
      connectionState: props.mapParams.connectionState,
    };

    if (pause.current) {
      pause.current = false;
      if (frameCallback.current) {
        requestAnimationFrame(frameCallback.current);
      }
    }
  }, [props.mapParams.location, props.mapParams.connectionState]);

  useEffect(() => {
    const resizeCallback = () => {
      if (canvasRef.current) {
        updateCanvasSize(canvasRef.current);
      }
    };

    addEventListener('resize', resizeCallback);
    return () => removeEventListener('resize', resizeCallback);
  }, [updateCanvasSize]);

  // TODO: Don't show location dot when spinner is showing
  return (
    <StyledCanvas ref={combinedCanvasRef} id="glcanvas" width={canvasWidth} height={canvasHeight} />
  );
}

type AnimationFrameCallback = (now: number, newParams?: MapParams) => void;

function getAnimationFramCallback(
  canvas: HTMLCanvasElement,
  startingCoordinate: Coordinate,
  connectionState: ConnectionState,
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
