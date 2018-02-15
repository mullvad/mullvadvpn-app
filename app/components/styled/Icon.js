// @flow
import React from 'react';
import { Component } from 'reactxp';
import Img from '../Img';

import { createViewStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    icon:{
      position: 'absolute',
      alignSelf: 'flex-end',
    },
  }),
};

export class Icon extends Component {
  render() {
    const width = this.props.width || 7;
    const height = this.props.height || 12;
    const source = this.props.source || 'icon-chevron';
    return (
      <Img style={[ styles.icon,
        {width,
          height},
        this.props.style]}
      source={source}
      tintColor='currentColor'/>);
  }
}