import { mat4 } from 'gl-matrix';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';

import log from '../../shared/logging';
import { useAppContext } from '../context';
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

type AnimationFrameCallback = (now: number, newParams?: MapParams) => void;

export default function Map() {
  const connection = useSelector((state) => state.connection);

  const location = useMemo<Coordinate | undefined>(() => {
    const { latitude, longitude } = connection;
    return typeof latitude === 'number' && typeof longitude === 'number'
      ? new Coordinate(latitude, longitude)
      : undefined;
  }, [connection.latitude, connection.longitude]);

  const connectionState = useMemo<ConnectionState>(() => {
    switch (connection.status.state) {
      case 'connected':
        return ConnectionState.connected;
      case 'disconnected':
        return ConnectionState.disconnected;
      default:
        return ConnectionState.noMarker;
    }
  }, [connection.status]);

  if (location === undefined) {
    return null;
  } else {
    return <MapInner location={location} connectionState={connectionState} />;
  }
}

function MapInner(props: MapParams) {
  const { getMapData } = useAppContext();

  // Callback that should be passed to requestAnimationFrame. This is initialized after the canvas
  // has been rendered.
  const animationFrameCallback = useRef<AnimationFrameCallback>();
  // When location or connection state changes it's stored here until passed to 3dmap
  const newParams = useRef<MapParams>();

  // This is set to true when rendering should be paused
  const pause = useRef<boolean>(false);

  const canvasRef = useRef<HTMLCanvasElement>();
  const [canvasWidth, setCanvasWidth] = useState(window.innerWidth);
  const [canvasHeight, setCanvasHeight] = useState(493);

  const scaleFactor = window.devicePixelRatio;

  const updateCanvasSize = useCallback((canvas: HTMLCanvasElement) => {
    const scaleFactor = window.devicePixelRatio;
    const canvasRect = canvas.getBoundingClientRect();

    canvas.width = Math.floor(canvasRect.width * scaleFactor);
    canvas.height = Math.floor(canvasRect.height * scaleFactor);

    setCanvasWidth(canvasRect.width);
    setCanvasHeight(canvasRect.height);
  }, []);

  // This is called when the canvas has been rendered the first time and initializes the gl context
  // and the map.
  const canvasCallback = useCallback(async (canvas: HTMLCanvasElement | null) => {
    if (!canvas) {
      return;
    }

    updateCanvasSize(canvas);

    const gl = canvas.getContext('webgl2', { antialias: true })!;
    setGlOptions(gl);

    const projectionMatrix = getProjectionMatrix(gl);

    const map = new GLMap(
      gl,
      await getMapData(),
      props.location,
      props.connectionState,
      () => (pause.current = true),
    );

    // Function to be used when calling requestAnimationFrame
    animationFrameCallback.current = (now: number) => {
      now *= 0.001; // convert to seconds

      // Propagate location change to the map
      if (newParams.current) {
        map.setLocation(newParams.current.location, newParams.current.connectionState, now);
        newParams.current = undefined;
      }

      drawScene(gl, map, projectionMatrix, now);

      // Stops rendering if pause is true. This happens when there is no ongoing movements
      if (!pause.current) {
        requestAnimationFrame(animationFrameCallback.current!);
      }
    };

    requestAnimationFrame(animationFrameCallback.current);
  }, []);

  // Set new params when the location or connection state has changed, and unpause if paused
  useEffect(() => {
    newParams.current = {
      location: props.location,
      connectionState: props.connectionState,
    };

    if (pause.current) {
      pause.current = false;
      if (animationFrameCallback.current) {
        requestAnimationFrame(animationFrameCallback.current);
      }
    }
  }, [props.location, props.connectionState]);

  // Resize canvas if window size changes
  useEffect(() => {
    const resizeCallback = () => {
      if (canvasRef.current) {
        updateCanvasSize(canvasRef.current);
      }
    };

    addEventListener('resize', resizeCallback);
    return () => removeEventListener('resize', resizeCallback);
  }, [updateCanvasSize]);

  // Log new scale factor if it changes
  useEffect(() => log.verbose('Map canvas scale factor:', scaleFactor), [scaleFactor]);

  const combinedCanvasRef = useCombinedRefs(canvasRef, canvasCallback);

  return (
    <StyledCanvas
      ref={combinedCanvasRef}
      id="glcanvas"
      width={Math.floor(canvasWidth * scaleFactor)}
      height={Math.floor(canvasHeight * scaleFactor)}
    />
  );
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

function drawScene(gl: WebGL2RenderingContext, map: GLMap, projectionMatrix: mat4, now: number) {
  gl.clearColor(10 / 255, 25 / 255, 35 / 255, 1); // Clear to black, fully opaque
  gl.clearDepth(1.0); // Clear everything
  gl.enable(gl.DEPTH_TEST); // Enable depth testing
  gl.depthFunc(gl.LEQUAL); // Near things obscure far things

  // Clear the canvas before we start drawing on it.
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  map.draw(projectionMatrix, now);
}
