// @flow
import React, { Component }  from 'react';
import { StyleSheet } from 'react-native';
import { Image, Styles } from 'reactxp';

export default class Img extends Component {
  props: {
    source: string,
    style: Object,
    tintColor?: string,
    height?: number,
    width?:number,
  };

  render(){
    const width = this.props.width || 7;
    const height = this.props.height || 12;
    const source = this.props.source || 'icon-chevron';
    const tintColor = this.props.tintColor || 'currentColor';

    if (tintColor === 'currentColor' && this.props.style) {
      const { color: tint, ...otherStyles } = StyleSheet.flatten(this.props.style);
      return(
        <Image style={Styles.createViewStyle({ ...otherStyles, tintColor: tint, height: height, width: width }, false)} source={ source }/>
      );
    } else {
      return(
        <Image style={Styles.createViewStyle({ ...this.props.style, height: height, width: width }, false)} source={ source }/>
      );
    }
  }
}
