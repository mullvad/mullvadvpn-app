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
    var flattenedStyle = StyleSheet.flatten(style);

    if (tintColor === 'currentColor' && flattenedStyle && flattenedStyle.color) {
      const tint = flattenedStyle.color;
      delete flattenedStyle.color;
      return(
        <Image style={[ flattenedStyle, { tintColor: tint } ]} source={ source }/>
      );
    } else {
      return(
        <Image style={ style } source={ source }/>
      );
    }
  }
}
