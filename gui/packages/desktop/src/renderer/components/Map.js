// @flow

import * as React from 'react';
import { Component, View } from 'reactxp';

import SvgMap from './SvgMap';

export type MapProps = {
  center: [number, number], // longitude, latitude
  offset: [number, number], // offset [x, y] from the center of the map
  zoomLevel: 'high' | 'medium' | 'low',
  showMarker: boolean,
  markerStyle: 'secure' | 'unsecure',
  style: Object,
};

type MapState = {
  bounds: {
    width: number,
    height: number,
  },
};

export default class Map extends Component<MapProps, MapState> {
  state = {
    bounds: {
      width: 0,
      height: 0,
    },
  };

  render() {
    const { width, height } = this.state.bounds;
    const readyToRenderTheMap = width > 0 && height > 0;
    return (
      <View style={this.props.style} onLayout={this._onLayout}>
        {readyToRenderTheMap && (
          <SvgMap
            width={width}
            height={height}
            center={this.props.center}
            offset={this.props.offset}
            zoomLevel={this._zoomLevel(this.props.zoomLevel)}
            showMarker={this.props.showMarker}
            markerImagePath={this._markerImage(this.props.markerStyle)}
          />
        )}
      </View>
    );
  }

  shouldComponentUpdate(nextProps: MapProps, nextState: MapState) {
    const oldProps = this.props;
    const oldState = this.state;
    return (
      oldProps.center[0] !== nextProps.center[0] ||
      oldProps.center[1] !== nextProps.center[1] ||
      oldProps.offset[0] !== nextProps.offset[0] ||
      oldProps.offset[1] !== nextProps.offset[1] ||
      oldProps.zoomLevel !== nextProps.zoomLevel ||
      oldProps.showMarker !== nextProps.showMarker ||
      oldProps.markerStyle !== nextProps.markerStyle ||
      oldState.bounds.width !== nextState.bounds.width ||
      oldState.bounds.height !== nextState.bounds.height
    );
  }

  _onLayout = (layoutInfo) => {
    this.setState({
      bounds: {
        width: layoutInfo.width,
        height: layoutInfo.height,
      },
    });
  };

  // TODO: Remove zoom level in favor of center + coordinate span
  _zoomLevel(variant: $PropertyType<MapProps, 'zoomLevel'>) {
    switch (variant) {
      case 'high':
        return 1;
      case 'medium':
        return 20;
      case 'low':
        return 40;
      default:
        throw new Error(`Invalid enumeration type: ${variant}`);
    }
  }

  _markerImage(style: $PropertyType<MapProps, 'markerStyle'>) {
    switch (style) {
      case 'secure':
        return '../assets/images/location-marker-secure.svg';
      case 'unsecure':
        return '../assets/images/location-marker-unsecure.svg';
      default:
        throw new Error(`Invalid enumeration type: ${style}`);
    }
  }
}
