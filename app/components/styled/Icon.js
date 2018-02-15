// @flow
import React from 'react';
import { Component } from 'reactxp';
import Img from '../Img';

export class Icon extends Component {
  render() {
    const width = this.props.width || 7;
    const height = this.props.height || 12;
    const source = this.props.source || 'icon-chevron';
    return (
      <Img style={[{width,
        height},
      this.props.style]}
      source={source}
      tintColor='currentColor'/>);
  }
}