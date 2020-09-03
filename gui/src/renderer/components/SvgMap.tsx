import { geoMercator, GeoProjection } from 'd3-geo';
import rbush from 'rbush';
import * as React from 'react';
import { ComposableMap, Geographies, Geography, Marker, ZoomableGroup } from 'react-simple-maps';

import geographyData from '../../../assets/geo/geometry.json';
import statesProvincesLinesData from '../../../assets/geo/states-provinces-lines.json';

import geometryTreeData from '../../../assets/geo/geometry.rbush.json';
import statesProvincesLinesTreeData from '../../../assets/geo/states-provinces-lines.rbush.json';

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

function getProjection(
  width: number,
  height: number,
  offsetX: number,
  offsetY: number,
  scale: number,
) {
  return geoMercator()
    .scale(scale)
    .translate([offsetX + width / 2, offsetY + height / 2])
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

function shouldInvalidateState(oldProps: IProps, nextProps: IProps) {
  return (
    oldProps.width !== nextProps.width ||
    oldProps.height !== nextProps.height ||
    oldProps.center[0] !== nextProps.center[0] ||
    oldProps.center[1] !== nextProps.center[1] ||
    oldProps.offset[0] !== nextProps.offset[0] ||
    oldProps.offset[1] !== nextProps.offset[1] ||
    oldProps.zoomLevel !== nextProps.zoomLevel
  );
}

function getNextState(prevState: IState | null, nextProps: IProps): IState {
  const { width, height, center, offset, zoomLevel } = nextProps;
  const viewportBboxes = prevState === null ? [] : prevState.viewportBboxes;

  const projection = getProjection(width, height, offset[0], offset[1], projectionConfig.scale);
  const zoomCenter = getZoomCenter(center, offset, projection, zoomLevel);

  const viewportBbox = getViewportGeoBoundingBox(zoomCenter, width, height, projection, zoomLevel);
  viewportBboxes.push(viewportBbox);

  const combinedViewportBboxMatch = {
    minX: Math.min(...viewportBboxes.map((viewportBbox) => viewportBbox[0])),
    minY: Math.min(...viewportBboxes.map((viewportBbox) => viewportBbox[1])),
    maxX: Math.max(...viewportBboxes.map((viewportBbox) => viewportBbox[2])),
    maxY: Math.max(...viewportBboxes.map((viewportBbox) => viewportBbox[3])),
  };

  const visibleGeometry = geometryTree.search(combinedViewportBboxMatch);
  const visibleStatesProvincesLines = provincesStatesLinesTree.search(combinedViewportBboxMatch);

  return {
    zoomCenter,
    zoomLevel,
    visibleGeometry,
    visibleStatesProvincesLines,
    viewportBboxes,
  };
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

interface IState {
  zoomCenter: [number, number];
  zoomLevel: number;
  visibleGeometry: IGeometryLeaf[];
  visibleStatesProvincesLines: IProvinceAndStateLineLeaf[];
  // combine previous and current viewports to get the rough area of transition.
  viewportBboxes: BBox[];
}

// @TODO: Calculate zoom level based on (center + span) (aka MKCoordinateSpan)
export default class SvgMap extends React.Component<IProps, IState> {
  public state: IState = getNextState(null, this.props);

  public UNSAFE_componentWillReceiveProps(nextProps: IProps) {
    if (shouldInvalidateState(this.props, nextProps)) {
      this.setState((prevState) => getNextState(prevState, nextProps));
    }
  }

  public shouldComponentUpdate(nextProps: IProps, nextState: IState) {
    return (
      this.props.width !== nextProps.width ||
      this.props.height !== nextProps.height ||
      this.props.center[0] !== nextProps.center[0] ||
      this.props.center[1] !== nextProps.center[1] ||
      this.props.offset[0] !== nextProps.offset[0] ||
      this.props.offset[1] !== nextProps.offset[1] ||
      this.props.zoomLevel !== nextProps.zoomLevel ||
      this.props.showMarker !== nextProps.showMarker ||
      this.props.markerImagePath !== nextProps.markerImagePath ||
      this.state.zoomCenter !== nextState.zoomCenter ||
      this.state.zoomLevel !== nextState.zoomLevel
    );
  }

  public render() {
    const mapStyle = {
      width: '100%',
      height: '100%',
      backgroundColor: '#192e45',
    };

    const zoomableGroupStyle = {
      transition: `transform ${MOVE_SPEED}ms ease-in-out`,
    };

    const geographyStyle = mergeRsmStyle({
      default: {
        fill: '#294d73',
        stroke: '#192e45',
        strokeWidth: `${1 / this.state.zoomLevel}`,
      },
    });

    const stateProvinceLineStyle = mergeRsmStyle({
      default: {
        fill: 'transparent',
        stroke: '#192e45',
        strokeWidth: `${1 / this.state.zoomLevel}`,
      },
    });

    const markerStyle = mergeRsmStyle({
      default: {
        transition: `transform ${MOVE_SPEED}ms ease-in-out`,
      },
    });

    // disable CSS transition when moving between locations
    // by using the different "key"
    const userMarker = this.props.showMarker && (
      <Marker
        key={`user-location-${this.props.center.join('-')}`}
        coordinates={this.props.center}
        style={markerStyle}>
        <image x="-6" y="-6" width="12" xlinkHref={this.props.markerImagePath} />
      </Marker>
    );

    return (
      <ComposableMap
        width={this.props.width}
        height={this.props.height}
        style={mapStyle}
        projection={
          // Workaround for incorrect type definition in @types/react-simple-maps.
          /* @ts-ignore */
          getProjection(
            this.props.height,
            this.props.width,
            this.props.offset[0],
            this.props.offset[1],
            projectionConfig.scale,
          ) as () => GeoProjection
        }
        projectionConfig={projectionConfig}>
        <ZoomableGroup
          center={this.state.zoomCenter}
          zoom={this.state.zoomLevel}
          onTransitionEnd={this.removeOldViewportBboxes}
          style={zoomableGroupStyle}>
          <Geographies geography={geographyData}>
            {({ geographies }) => {
              return this.state.visibleGeometry.map(({ id }) => (
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
              return this.state.visibleStatesProvincesLines.map(({ id }) => (
                <Geography
                  key={id}
                  geography={geographies[parseInt(id, 10)]}
                  style={stateProvinceLineStyle}
                />
              ));
            }}
          </Geographies>
          {[userMarker]}
        </ZoomableGroup>
      </ComposableMap>
    );
  }

  private removeOldViewportBboxes = () => {
    this.setState((state) => ({ viewportBboxes: state.viewportBboxes.slice(-1) }));
  };
}
