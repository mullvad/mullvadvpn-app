// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { createTextStyles } from '../../lib/styles';

const styles = {
  ...createTextStyles({
    subtext:{
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '800',
      flex: 0,
      textAlign: 'right',
    },
  })
};

export class SubText extends Component {
  render() {
    return (
      <Text style={[ styles.subtext, this.props.style ]}>
        {this.props.children}
      </Text>
    );
  }
}