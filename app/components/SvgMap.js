// @flow

import React, { Component } from 'react';
import { ComposableMap, ZoomableGroup, Geographies, Geography, Markers, Marker } from 'react-simple-maps';

import { geoTimes } from 'd3-geo-projection';
import rbush from 'rbush';

import geographyData from '../assets/geo/geometry.json';
import statesProvincesLinesData from '../assets/geo/states-provinces-lines.json';

import countryTreeData from '../assets/geo/countries.rbush.json';
import cityTreeData from '../assets/geo/cities.rbush.json';
import geometryTreeData from '../assets/geo/geometry.rbush.json';
import statesProvincesLinesTreeData from '../assets/geo/states-provinces-lines.rbush.json';

const countryTree = rbush().fromJSON(countryTreeData);
const cityTree = rbush().fromJSON(cityTreeData);
const geometryTree = rbush().fromJSON(geometryTreeData);
const provincesStatesLinesTree = rbush().fromJSON(statesProvincesLinesTreeData);

type BBox = [number, number, number, number];

export type SvgMapProps = {
  width: number,
  height: number,
  center: [number, number], // longitude, latitude
  offset: [number, number], // [x, y] in points
  zoomLevel: number,
  showMarker: boolean,
  markerImagePath: string,
};

type SvgMapState = {
  zoomCenter: [number, number],
  zoomLevel: number,
  visibleCities: Array<Object>,
  visibleCountries: Array<Object>,
  visibleGeometry: Array<Object>,
  visibleStatesProvincesLines: Array<Object>,
  viewportBbox: BBox,
};

const MOVE_SPEED = 2000;

// @TODO: Calculate zoom level based on (center + span) (aka MKCoordinateSpan)
export default class SvgMap extends Component {
  props: SvgMapProps;
  state: SvgMapState = {
    zoomCenter: [0, 0],
    zoomLevel: 1,
    visibleCities: [],
    visibleCountries: [],
    visibleGeometry: [],
    visibleStatesProvincesLines: [],
    viewportBbox: [0, 0, 0, 0],
  };

  _projectionConfig = {
    scale: 160
  };

  constructor(props: SvgMapProps) {
    super(props);

    this.state = this._getNextState(null, props);
  }

  componentWillReceiveProps(nextProps: SvgMapProps) {
    if(this._shouldInvalidateState(nextProps)) {
      this.setState(prevState => this._getNextState(prevState, nextProps));
    }
  }

  shouldComponentUpdate(nextProps: SvgMapProps, nextState: SvgMapState) {
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

  render() {
    const mapStyle = {
      width: '100%',
      height: '100%',
      backgroundColor: '#192e45',
    };

    const zoomableGroupStyle = {
      transition: `transform ${MOVE_SPEED}ms ease-in-out`
    };

    const geographyStyle = this._mergeRsmStyle({
      default: {
        fill: '#294d73',
        stroke: '#192e45',
        strokeWidth: `${1 / this.state.zoomLevel}`,
      }
    });

    const stateProvinceLineStyle = this._mergeRsmStyle({
      default: {
        fill: 'transparent',
        stroke: '#192e45',
        strokeWidth: `${1 / this.state.zoomLevel}`,
      }
    });

    const markerStyle = this._mergeRsmStyle({
      default: {
        transition: `transform ${MOVE_SPEED}ms ease-in-out`,
      },
    });

    // disable CSS transition when moving between locations
    // by using the different "key"
    const userMarker = this.props.showMarker && (
      <Marker key={ `user-location-${ this.props.center.join('-') }` }
        marker={{ coordinates: this.props.center }}
        style={ markerStyle }>
        <image x="-30" y="-30" href={ this.props.markerImagePath } />
      </Marker>
    );

    const countryMarkers = this.state.visibleCountries.map(item => (
      <Marker key={ `country-${item.id}` }
        marker={{ coordinates: item.geometry.coordinates }}
        style={ markerStyle }>
        <text fill="rgba(255,255,255,.6)" fontSize="22" textAnchor="middle">
          { item.properties.name }
        </text>
      </Marker>
    ));

    const cityMarkers = this.state.visibleCities.map(item => (
      <Marker key={ `city-${item.id}` }
        marker={{ coordinates: item.geometry.coordinates }}
        style={ markerStyle }>
        <circle r="2" fill="rgba(255,255,255,.6)" />
        <text x="0" y="-10" fill="rgba(255,255,255,.6)" fontSize="16" textAnchor="middle">
          { item.properties.name }
        </text>
      </Marker>
    ));

    return (
      <ComposableMap
        width={ this.props.width }
        height={ this.props.height }
        style={ mapStyle }
        projection={ this._getProjection }
        projectionConfig={ this._projectionConfig }>
        <ZoomableGroup
          center={ this.state.zoomCenter }
          zoom={ this.state.zoomLevel }
          disablePanning={ false }
          style={ zoomableGroupStyle }>
          <Geographies geography={ geographyData } disableOptimization={ true }>
            {(geographies, projection) => {
              return this.state.visibleGeometry.map(({ id }) => (
                <Geography
                  key={ id }
                  geography={ geographies[id] }
                  projection={ projection }
                  style={ geographyStyle } />
              ));
            }}
          </Geographies>
          <Geographies geography={ statesProvincesLinesData } disableOptimization={ true }>
            {(geographies, projection) => {
              return this.state.visibleStatesProvincesLines.map(({ id }) => (
                <Geography
                  key={ id }
                  geography={ geographies[id] }
                  projection={ projection }
                  style={ stateProvinceLineStyle } />
              ));
            }}
          </Geographies>
          <Markers>
            { [...countryMarkers, ...cityMarkers, userMarker] }
          </Markers>
        </ZoomableGroup>
      </ComposableMap>
    );
  }

  _mergeRsmStyle(style: Object) {
    const defaultStyle = style.default || {};
    return {
      default: defaultStyle,
      hover: style.hover || defaultStyle,
      pressed: style.pressed || defaultStyle
    };
  }

  _getProjection(width: number, height: number, config: {
    scale?: number,
    xOffset?: number,
    yOffset?: number,
    rotation?: [number, number, number],
    precision?: number,
  }) {
    const scale = config.scale || 160;
    const xOffset = config.xOffset || 0;
    const yOffset = config.yOffset || 0;
    const rotation = config.rotation || [0, 0, 0];
    const precision = config.precision || 0.1;

    return geoTimes()
      .scale(scale)
      .translate([ xOffset + width / 2, yOffset + height / 2 ])
      .rotate(rotation)
      .precision(precision);
  }

  _getZoomCenter(
    center: [number, number],
    offset: [number, number],
    projection: Function,
    zoom: number
  ) {
    const pos = projection(center);
    return projection.invert([
      pos[0] + offset[0] / zoom,
      pos[1] + offset[1] / zoom
    ]);
  }

  _getViewportGeoBoundingBox(
    centerCoordinate: [number, number],
    width: number, height: number,
    projection: Function,
    zoom: number
  ) {
    const center = projection(centerCoordinate);
    const halfWidth = width * 0.5 / zoom;
    const halfHeight = height * 0.5 / zoom;

    const northWest = projection.invert([center[0] - halfWidth, center[1] - halfHeight]);
    const southEast = projection.invert([center[0] + halfWidth, center[1] + halfHeight]);

    // normalize to [minX, minY, maxX, maxY]
    return [
      Math.min(northWest[0], southEast[0]),
      Math.min(northWest[1], southEast[1]),
      Math.max(northWest[0], southEast[0]),
      Math.max(northWest[1], southEast[1]),
    ];
  }

  _shouldInvalidateState(nextProps: SvgMapProps) {
    const oldProps = this.props;
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

  _getNextState(prevState: ?SvgMapState, nextProps: SvgMapProps): SvgMapState {
    const { width, height, center, offset, zoomLevel } = nextProps;

    const projection = this._getProjection(width, height, this._projectionConfig);
    const zoomCenter = this._getZoomCenter(center, offset, projection, zoomLevel);
    const viewportBbox = this._getViewportGeoBoundingBox(zoomCenter, width, height, projection, zoomLevel);

    const viewportBboxMatch = {
      minX: viewportBbox[0], minY: viewportBbox[1],
      maxX: viewportBbox[2], maxY: viewportBbox[3],
    };

    // combine previous and current viewports to get the rough area of transition
    const combinedViewportBboxMatch = prevState ? {
      minX: Math.min(viewportBbox[0], prevState.viewportBbox[0]),
      minY: Math.min(viewportBbox[1], prevState.viewportBbox[1]),
      maxX: Math.max(viewportBbox[2], prevState.viewportBbox[2]),
      maxY: Math.max(viewportBbox[3], prevState.viewportBbox[3]),
    } : {
      minX: viewportBbox[0],
      minY: viewportBbox[1],
      maxX: viewportBbox[2],
      maxY: viewportBbox[3],
    };

    const visibleCountries = zoomLevel < 5 || zoomLevel > 20 ? [] : countryTree.search(viewportBboxMatch);
    const visibleCities = zoomLevel >= 40 ? cityTree.search(viewportBboxMatch) : [];
    const visibleGeometry = geometryTree.search(combinedViewportBboxMatch);
    const visibleStatesProvincesLines = provincesStatesLinesTree.search(combinedViewportBboxMatch);

    return {
      zoomCenter,
      zoomLevel,
      visibleCities,
      visibleCountries,
      visibleGeometry,
      visibleStatesProvincesLines,
      viewportBbox,
    };
  }
}
