import * as React from 'react';
import SvgMap from './SvgMap';

// Higher zoom level is more zoomed in
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
  className?: string;
}

interface IState {
  bounds: {
    width: number;
    height: number;
  };
}

export default class Map extends React.Component<IProps, IState> {
  public state: IState = {
    bounds: {
      width: 0,
      height: 0,
    },
  };

  private containerRef = React.createRef<HTMLDivElement>();

  public render() {
    const { width, height } = this.state.bounds;
    const readyToRenderTheMap = width > 0 && height > 0;
    return (
      <div className={this.props.className} ref={this.containerRef}>
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
      </div>
    );
  }

  public componentDidMount() {
    this.updateBounds();
  }

  public componentDidUpdate() {
    this.updateBounds();
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

  private updateBounds() {
    const containerRect = this.containerRef.current?.getBoundingClientRect();
    if (containerRect) {
      this.setState((state) => {
        if (
          containerRect.width === state.bounds.width &&
          containerRect.height === state.bounds.height
        ) {
          return null;
        } else {
          return {
            bounds: {
              width: containerRect.width,
              height: containerRect.height,
            },
          };
        }
      });
    }
  }

  // TODO: Remove zoom level in favor of center + coordinate span
  // TODO: Zoomlevels below 2.22 makes australia invisible
  private zoomLevel(variant: ZoomLevel) {
    switch (variant) {
      case ZoomLevel.low:
        return 1;
      case ZoomLevel.medium:
        return 2.22;
      case ZoomLevel.high:
        return 5;
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
