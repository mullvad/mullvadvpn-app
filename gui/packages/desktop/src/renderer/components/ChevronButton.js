// @flow

import * as React from 'react';
import { Component, Styles } from 'reactxp';
import * as Cell from './Cell';
import { colors } from '../../config';

type Props = {
  up: boolean,
  onPress: () => void,
};

const style = Styles.createViewStyle({
  flex: 0,
  alignSelf: 'stretch',
  justifyContent: 'center',
  paddingRight: 16,
  paddingLeft: 16,
});

export default class ChevronButton extends Component<Props> {
  render() {
    return (
      <Cell.Icon
        style={[style, this.props.style]}
        tintColor={colors.white80}
        tintHoverColor={colors.white}
        onPress={this.props.onPress}
        source={this.props.up ? 'icon-chevron-up' : 'icon-chevron-down'}
        height={24}
        width={24}
      />
    );
  }
}
