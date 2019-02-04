import * as React from 'react';
import { Component, Types, View } from 'reactxp';

import SvgMap from './SvgMap';

export enum ZoomLevel {
  high,
  medium,
  low,
}

export enum MarkerStyle {
  secure,
  unsecure,
}

interface IProps {
  center: [number, number]; // longitude, latitude
  offset: [number, number]; // offset [x, y] from the center of the map
  zoomLevel: ZoomLevel;
  showMarker: boolean;
  markerStyle: MarkerStyle;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

interface IState {
  bounds: {
    width: number;
    height: number;
  };
}

export default class Map extends Component<IProps, IState> {
  public state: IState = {
    bounds: {
      width: 0,
      height: 0,
    },
  };

  public render() {
    const { width, height } = this.state.bounds;
    const readyToRenderTheMap = width > 0 && height > 0;
    return (
      <View style={this.props.style} onLayout={this.onLayout}>
        {readyToRenderTheMap && (
          <SvgMap
            width={width}
            height={height}
            center={this.props.center}
            offset={this.props.offset}
            zoomLevel={this.zoomLevel(this.props.zoomLevel)}
            showMarker={this.props.showMarker}
            markerImagePath={this.markerImage(this.props.markerStyle)}
          />
        )}
      </View>
    );
  }

  public shouldComponentUpdate(nextProps: IProps, nextState: IState) {
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

  private onLayout = (layoutInfo: Types.ViewOnLayoutEvent) => {
    this.setState({
      bounds: {
        width: layoutInfo.width,
        height: layoutInfo.height,
      },
    });
  };

  // TODO: Remove zoom level in favor of center + coordinate span
  private zoomLevel(variant: ZoomLevel) {
    switch (variant) {
      case ZoomLevel.high:
        return 1;
      case ZoomLevel.medium:
        return 20;
      case ZoomLevel.low:
        return 40;
    }
  }

  private markerImage(style: MarkerStyle): string {
    switch (style) {
      case MarkerStyle.secure:
        return '../../assets/images/location-marker-secure.svg';
      case MarkerStyle.unsecure:
        return '../../assets/images/location-marker-unsecure.svg';
    }
  }
}
