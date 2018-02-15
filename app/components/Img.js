// @flow
import React from 'react';
import { View, Component } from 'reactxp';

export default class Img extends Component {
  props: {
    source: string,
    tintColor?: string
  };

  render() {
    const { source, ...otherProps } = this.props;
    const tintColor = this.props.tintColor;
    const url = './assets/images/' + source + '.svg';
    let image;

    if(tintColor) {
      image = (
        <div style={{
          WebkitMaskImage: `url('${url}')`,
          WebkitMaskRepeat: 'no-repeat',
          backgroundColor: tintColor,
          lineHeight: 0,
        }}>
          <img src={ url } style={{
            visibility: 'hidden',
          }} />
        </div>
      );
    } else {
      image = (
        <img src={ url } />
      );
    }

    return (
      <View { ...otherProps }>
        { image }
      </View>);
  }
}
