import { geoMercator, GeoProjection } from 'd3-geo';
import rbush from 'rbush';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ComposableMap, Geographies, Geography, Marker, ZoomableGroup } from 'react-simple-maps';

import geographyData from '../../../assets/geo/geometry.json';
import statesProvincesLinesData from '../../../assets/geo/states-provinces-lines.json';

import geometryTreeData from '../../../assets/geo/geometry.rbush.json';
import statesProvincesLinesTreeData from '../../../assets/geo/states-provinces-lines.rbush.json';
import { useScheduler } from '../../shared/scheduler';

interface IGeometryLeaf extends rbush.BBox {
  id: string;
}

interface IProvinceAndStateLineLeaf extends rbush.BBox {
  id: string;
}

const geometryTree = rbush<IGeometryLeaf>().fromJSON(geometryTreeData);
const provincesStatesLinesTree = rbush<IProvinceAndStateLineLeaf>().fromJSON(
  statesProvincesLinesTreeData,
);

type BBox = [number, number, number, number];

const MOVE_SPEED = 2000;

const mapStyle = {
  width: '100%',
  height: '100%',
  backgroundColor: '#192e45',
};
const zoomableGroupStyle = {
  transition: `transform ${MOVE_SPEED}ms ease-out`,
};

function getMarkerImageStyle(zoom: number) {
  return {
    width: '60px',
    transform: `translate(${-30 / zoom}px, ${-30 / zoom}px) scale(${1 / zoom})`,
    transition: `transform ${MOVE_SPEED}ms ease-out`,
  };
}

const geographyStyle = mergeRsmStyle({
  default: {
    fill: '#294d73',
    stroke: '#192e45',
    strokeWidth: 0.2,
  },
});

const stateProvinceLineStyle = mergeRsmStyle({
  default: {
    fill: 'transparent',
    stroke: '#192e45',
    strokeWidth: 0.2,
  },
});

const projectionConfig = {
  scale: 160,
};

function mergeRsmStyle(style: {
  default?: React.CSSProperties;
  hover?: React.CSSProperties;
  pressed?: React.CSSProperties;
}) {
  const defaultStyle = style.default || {};
  return {
    default: defaultStyle,
    hover: style.hover || defaultStyle,
    pressed: style.pressed || defaultStyle,
  };
}

function getProjection(width: number, height: number, offset: [number, number], scale: number) {
  return geoMercator()
    .scale(scale)
    .translate([offset[0] + width / 2, offset[1] + height / 2])
    .precision(0.1);
}

function getZoomCenter(
  center: [number, number],
  offset: [number, number],
  projection: GeoProjection,
  zoom: number,
): [number, number] {
  const pos = projection(center)!;
  return projection.invert!([pos[0] + offset[0] / zoom, pos[1] + offset[1] / zoom])!;
}

function getViewportGeoBoundingBox(
  centerCoordinate: [number, number],
  width: number,
  height: number,
  projection: GeoProjection,
  zoom: number,
): BBox {
  const center = projection(centerCoordinate)!;
  const halfWidth = (width * 0.5) / zoom;
  const halfHeight = (height * 0.5) / zoom;

  const northWest = projection.invert!([center[0] - halfWidth, center[1] - halfHeight])!;
  const southEast = projection.invert!([center[0] + halfWidth, center[1] + halfHeight])!;

  // normalize to [minX, minY, maxX, maxY]
  return [
    Math.min(northWest[0], southEast[0]),
    Math.min(northWest[1], southEast[1]),
    Math.max(northWest[0], southEast[0]),
    Math.max(northWest[1], southEast[1]),
  ];
}

function getCombindedViewportBboxMatch(viewportBboxes: BBox[]) {
  return {
    minX: Math.min(...viewportBboxes.map((viewportBbox) => viewportBbox[0])),
    minY: Math.min(...viewportBboxes.map((viewportBbox) => viewportBbox[1])),
    maxX: Math.max(...viewportBboxes.map((viewportBbox) => viewportBbox[2])),
    maxY: Math.max(...viewportBboxes.map((viewportBbox) => viewportBbox[3])),
  };
}

function sameProps(prevProps: IProps, nextProps: IProps) {
  return (
    prevProps.width === nextProps.width &&
    prevProps.height === nextProps.height &&
    prevProps.center[0] === nextProps.center[0] &&
    prevProps.center[1] === nextProps.center[1] &&
    prevProps.offset[0] === nextProps.offset[0] &&
    prevProps.offset[1] === nextProps.offset[1] &&
    prevProps.zoomLevel === nextProps.zoomLevel &&
    prevProps.showMarker === nextProps.showMarker &&
    prevProps.markerImagePath === nextProps.markerImagePath
  );
}

function useViewportBboxes(
  center: [number, number],
  width: number,
  height: number,
  projection: GeoProjection,
  zoom: number,
): [BBox[], () => void] {
  const viewportBbox = useMemo(
    () => getViewportGeoBoundingBox(center, width, height, projection, zoom),
    [center, width, height, projection, zoom],
  );

  const prev = useRef<BBox[]>([]);
  const viewportBboxes = useMemo(() => [...prev.current, viewportBbox], [viewportBbox]);

  const keepLast = useCallback(() => {
    prev.current = prev.current.slice(-1);
  }, []);

  useEffect(() => {
    prev.current = [...viewportBboxes];
  }, [viewportBboxes]);

  return [viewportBboxes, keepLast];
}

function useVisibleGeometry(viewportBboxes: BBox[]) {
  const combinedViewportBboxMatch = useMemo(() => getCombindedViewportBboxMatch(viewportBboxes), [
    viewportBboxes,
  ]);
  const visibleGeometry = useMemo(() => geometryTree.search(combinedViewportBboxMatch), [
    combinedViewportBboxMatch,
  ]);
  const visibleStatesProvincesLines = useMemo(
    () => provincesStatesLinesTree.search(combinedViewportBboxMatch),
    [combinedViewportBboxMatch],
  );

  return [visibleGeometry, visibleStatesProvincesLines];
}

export interface IProps {
  width: number;
  height: number;
  center: [number, number]; // longitude, latitude
  offset: [number, number]; // [x, y] in points
  zoomLevel: number;
  showMarker: boolean;
  markerImagePath: string;
}

// @TODO: Calculate zoom level based on (center + span) (aka MKCoordinateSpan)
function SvgMap(props: IProps) {
  const { width, height, zoomLevel } = props;
  const center = useMemo(() => props.center, [...props.center]);
  const projection = useMemo(
    () => getProjection(width, height, props.offset, projectionConfig.scale),
    [width, height, ...props.offset, projectionConfig.scale],
  );
  const zoomCenter = useMemo(() => getZoomCenter(center, props.offset, projection, zoomLevel), [
    ...center,
    ...props.offset,
    projection,
    zoomLevel,
  ]);
  const [viewportBboxes, removeOldViewportBboxes] = useViewportBboxes(
    zoomCenter,
    width,
    height,
    projection,
    zoomLevel,
  );
  const [visibleGeometry, visibleStatesProvincesLines] = useVisibleGeometry(viewportBboxes);

  // react-simple-maps renders the map with zoom=1 the first render resulting in a transition from
  // 1 to zoomLevel when it immediately renders a second time. This makes sure that transitions are
  // disabled until after the second render.
  const [enableTransition, setEnableTransition] = useState(false);
  const enableTransitionScheduler = useScheduler();
  useEffect(() => enableTransitionScheduler.schedule(() => setEnableTransition(true)), []);

  const markerStyle = useMemo(
    () => mergeRsmStyle({ default: { display: props.showMarker ? undefined : 'none' } }),
    [props.showMarker],
  );
  const markerImageStyle = useMemo(() => getMarkerImageStyle(zoomLevel), [zoomLevel]);

  return (
    <ComposableMap
      width={width}
      height={height}
      style={mapStyle}
      projection={
        // Workaround for incorrect type definition in @types/react-simple-maps.
        /* @ts-ignore */
        projection as () => GeoProjection
      }
      projectionConfig={projectionConfig}>
      <ZoomableGroup
        center={zoomCenter}
        zoom={zoomLevel}
        onTransitionEnd={removeOldViewportBboxes}
        style={enableTransition ? zoomableGroupStyle : undefined}>
        <Geographies geography={geographyData}>
          {({ geographies }) => {
            return visibleGeometry.map(({ id }) => (
              <Geography
                key={id}
                geography={geographies[parseInt(id, 10)]}
                style={geographyStyle}
              />
            ));
          }}
        </Geographies>
        <Geographies geography={statesProvincesLinesData}>
          {({ geographies }) => {
            return visibleStatesProvincesLines.map(({ id }) => (
              <Geography
                key={id}
                geography={geographies[parseInt(id, 10)]}
                style={stateProvinceLineStyle}
              />
            ));
          }}
        </Geographies>
        <Marker coordinates={center} style={markerStyle}>
          <image style={markerImageStyle} xlinkHref={props.markerImagePath} />
        </Marker>
      </ZoomableGroup>
    </ComposableMap>
  );
}

export default React.memo(SvgMap, sameProps);
