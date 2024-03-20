import { useCallback, useEffect, useMemo, useRef } from 'react';
import styled from 'styled-components';

import { TunnelState } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import { useAppContext } from '../context';
import GlMap, { ConnectionState, Coordinate } from '../lib/3dmap';
import { useCombinedRefs, useRerenderer } from '../lib/utilityHooks';
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

export default function Map() {
  const connection = useSelector((state) => state.connection);
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);

  const hasLocationValue = hasLocation(connection);
  const location = useMemo<Coordinate | undefined>(() => {
    return hasLocationValue ? connection : defaultLocation;
  }, [hasLocationValue, connection.latitude, connection.longitude]);

  if (window.env.e2e) {
    return null;
  }

  const connectionState = getConnectionState(hasLocationValue, connection.status.state);

  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const animate = !reduceMotion && animateMap;

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

  // When location or connection state changes it's stored here until passed to 3dmap
  const newParams = useRef<MapParams>();

  // This is set to true when rendering should be paused
  const pause = useRef<boolean>(false);

  const mapRef = useRef<GlMap>();
  const canvasRef = useRef<HTMLCanvasElement>();
  const width = applyPixelRatio(canvasRef.current?.clientWidth ?? window.innerWidth);
  // This constant is used for the height the first frame that is rendered only.
  const height = applyPixelRatio(canvasRef.current?.clientHeight ?? 493);

  // Hack to rerender when window size changes or when ref is set.
  const [onSizeChange, sizeChangeCounter] = useRerenderer();

  const render = useCallback(() => requestAnimationFrame(animationFrameCallback), []);

  const animationFrameCallback = useCallback(
    (now: number) => {
      now *= 0.001; // convert to seconds

      // Propagate location change to the map
      if (newParams.current) {
        mapRef.current?.setLocation(
          newParams.current.location,
          newParams.current.connectionState,
          now,
          props.animate,
        );
        newParams.current = undefined;
      }

      mapRef.current?.draw(now);

      // Stops rendering if pause is true. This happens when there is no ongoing movements
      if (!pause.current) {
        render();
      }
    },
    [props.animate],
  );

  // This is called when the canvas has been rendered the first time and initializes the gl context
  // and the map.
  const canvasCallback = useCallback(async (canvas: HTMLCanvasElement | null) => {
    if (!canvas) {
      return;
    }

    onSizeChange();

    const gl = canvas.getContext('webgl2', { antialias: true })!;

    mapRef.current = new GlMap(
      gl,
      await getMapData(),
      props.location,
      props.connectionState,
      () => (pause.current = true),
    );

    render();
  }, []);

  // Set new params when the location or connection state has changed, and unpause if paused
  useEffect(() => {
    newParams.current = {
      location: props.location,
      connectionState: props.connectionState,
    };

    if (pause.current) {
      pause.current = false;
      render();
    }
  }, [props.location, props.connectionState]);

  useEffect(() => {
    mapRef.current?.updateViewport();
    render();
  }, [width, height, sizeChangeCounter]);

  // Resize canvas if window size changes
  useEffect(() => {
    addEventListener('resize', onSizeChange);
    return () => removeEventListener('resize', onSizeChange);
  }, []);

  useEffect(() => {
    const unsubscribe = window.ipc.window.listenScaleFactorChange(onSizeChange);
    return () => unsubscribe();
  }, []);

  // Log new scale factor if it changes
  useEffect(() => {
    log.verbose(`Map canvas scale factor: ${window.devicePixelRatio}, using: ${getPixelRatio()}`);
  }, [window.devicePixelRatio]);

  const combinedCanvasRef = useCombinedRefs(canvasRef, canvasCallback);

  return <StyledCanvas ref={combinedCanvasRef} width={width} height={height} />;
}

function getPixelRatio(): number {
  let pixelRatio = window.devicePixelRatio;

  // Wayland renders non-integer values as the next integer and then scales it back down.
  if (window.env.platform === 'linux') {
    pixelRatio = Math.ceil(pixelRatio);
  }

  return pixelRatio;
}

function applyPixelRatio(dimension: number): number {
  return Math.floor(dimension * getPixelRatio());
}
