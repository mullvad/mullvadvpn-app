import * as React from 'react';
import { Component, Styles, Types } from 'reactxp';
import { colors } from '../../config.json';
import * as Cell from './Cell';

interface IProps {
  up: boolean;
  onPress?: (event: Types.SyntheticEvent) => void;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

const style = Styles.createViewStyle({
  flex: 0,
  alignSelf: 'stretch',
  justifyContent: 'center',
  paddingRight: 16,
  paddingLeft: 16,
});

export default class ChevronButton extends Component<IProps> {
  public render() {
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
