import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';

import { TunnelState } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import { useAppContext } from '../context';
import GlMap, { ConnectionState, Coordinate } from '../lib/3dmap';
import { useCombinedRefs } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';

// Default to Gothenburg when we don't know the actual location.
const defaultLocation: Coordinate = { latitude: 57.70887, longitude: 11.97456 };

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

  const hasLocationValue = hasLocation(connection);
  const location = useMemo<Coordinate | undefined>(() => {
    return hasLocationValue ? connection : defaultLocation;
  }, [hasLocationValue, connection.latitude, connection.longitude]);

  const connectionState = getConnectionState(hasLocationValue, connection.status.state);

  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const animate = !reduceMotion;

  return (
    <MapInner
      location={location ?? defaultLocation}
      connectionState={connectionState}
      animate={animate}
    />
  );
}

function hasLocation(location: Partial<Coordinate>): location is Coordinate {
  return typeof location.latitude === 'number' && typeof location.longitude === 'number';
}

function getConnectionState(hasLocation: boolean, connectionState: TunnelState['state']) {
  if (!hasLocation) {
    return ConnectionState.noMarker;
  }

  switch (connectionState) {
    case 'connected':
      return ConnectionState.connected;
    case 'disconnected':
      return ConnectionState.disconnected;
    default:
      return ConnectionState.noMarker;
  }
}

interface MapInnerProps extends MapParams {
  animate: boolean;
}

function MapInner(props: MapInnerProps) {
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
  // This constant is used for the height the first frame that is rendered only.
  const [canvasHeight, setCanvasHeight] = useState(493);

  const updateCanvasSize = useCallback((canvas: HTMLCanvasElement) => {
    const canvasRect = canvas.getBoundingClientRect();

    canvas.width = applyScaleFactor(canvasRect.width);
    canvas.height = applyScaleFactor(canvasRect.height);

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

    const map = new GlMap(
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
        map.setLocation(
          newParams.current.location,
          newParams.current.connectionState,
          now,
          props.animate,
        );
        newParams.current = undefined;
      }

      map.draw(now);

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
  useEffect(() => log.verbose('Map canvas scale factor:', window.devicePixelRatio), [
    window.devicePixelRatio,
  ]);

  const combinedCanvasRef = useCombinedRefs(canvasRef, canvasCallback);

  return (
    <StyledCanvas
      ref={combinedCanvasRef}
      width={applyScaleFactor(canvasWidth)}
      height={applyScaleFactor(canvasHeight)}
    />
  );
}

function applyScaleFactor(dimension: number): number {
  const scaleFactor = window.devicePixelRatio;
  return Math.floor(dimension * scaleFactor);
}
