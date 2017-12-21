// @flow
import React from 'react';
import { View, Component } from 'reactxp';

export default class Img extends Component {
  props: {
    source: string,
    tintColor?: string
  };

  render() {
    const { source, tintColor, ...otherProps } = this.props;
    const url = './assets/images/' + source + '.svg';
    let image;

    if(tintColor) {
      image = (
        <div style={{
          WebkitMaskImage: `url('${url}')`,
          WebkitMaskRepeat: 'no-repeat',
          backgroundColor: tintColor,
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
