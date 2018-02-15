// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { createTextStyles } from '../../lib/styles';

const styles = {
  ...createTextStyles({
    label:{
      alignSelf: 'center',
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
    },
  })
};

export class Label extends Component {
  render() {
    return (
      <Text style={[ styles.label, this.props.style ]}>
        {this.props.children}
      </Text>
    );
  }
}