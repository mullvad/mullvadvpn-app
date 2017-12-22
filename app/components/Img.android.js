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

    if (tintColor === 'currentColor' && style) {
      const { color: tint, ...otherStyles } = StyleSheet.flatten(style);
      return(
        <Image style={[ otherStyles, { tintColor: tint } ]} source={ source }/>
      );
    } else {
      return(
        <Image style={ style } source={ source }/>
      );
    }
  }
}
