// @flow

import React, { Component } from 'react';
import ReactMapboxGl, { Marker } from 'react-mapbox-gl';
import { mapbox as mapboxConfig } from '../config';
import cheapRuler from 'cheap-ruler';

import type { Coordinate2d } from '../types';

const ReactMap = ReactMapboxGl({
  accessToken: mapboxConfig.accessToken,
  attributionControl: false,
  interactive: false,
});

export class Map extends Component {
  props: {
    animate: boolean,
    location: Coordinate2d,
    altitude: number,
    markerImagePath: string,
  }

  render() {

    const mapBounds = this.calculateMapBounds(this.props.location, this.props.altitude);

    const mapBoundsOptions = { offset: [0, -113], animate: this.props.animate };

    return <ReactMap style={ mapboxConfig.styleURL }
      containerStyle={{ height: '100%' }}
      fitBounds={ mapBounds }
      fitBoundsOptions={ mapBoundsOptions }>

      <Marker coordinates={ this.convertToMapCoordinate(this.props.location) } offset={ [0, -10] }>
        <img src={ this.props.markerImagePath } />
      </Marker>
    </ReactMap>;
  }

  calculateMapBounds(center: Coordinate2d, altitude: number): [Coordinate2d, Coordinate2d] {
    const bounds = cheapRuler(center[0], 'meters').bufferPoint(center, altitude);
    // convert [lat,lng] bounds to [lng,lat]
    return [ [bounds[1], bounds[0]], [bounds[3], bounds[2]] ];
  }

  convertToMapCoordinate(pos: Coordinate2d): Coordinate2d {
    // convert [lat,lng] bounds to [lng,lat]
    return [pos[1], pos[0]];
  }
}
