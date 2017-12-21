// @flow
import React, { Component }  from 'react';
import { StyleSheet } from 'react-native';
import { Image } from 'reactxp';

export default class Img extends Component {
  props: {
    source: string,
    style: Object,
    tintColor?: string
  };

  render(){
    const { source, tintColor, style } = this.props;
    var _style = StyleSheet.flatten(style);

    if (tintColor === 'currentColor' && Object.prototype.hasOwnProperty.call(_style, 'color')) {
      const tint = _style.color;
      delete _style.color;
      return(
        <Image style={[ _style, { tintColor: tint } ]} source={ source }/>
      );
    } else {
      return(
        <Image style={ style } source={ source }/>
      );
    }
  }
}
