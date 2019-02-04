import * as React from 'react';
import { Component, Styles } from 'reactxp';
import * as Cell from './Cell';
import RelayStatusIndicator from './RelayStatusIndicator';
import { colors } from '../../config.json';

type Props = {
  hostname: string;
  selected: boolean;
  onSelect?: () => void;
};

const styles = {
  base: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 60,
    backgroundColor: colors.blue20,
  }),
  selected: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
};

export default class RelayRow extends Component<Props> {
  shouldComponentUpdate(nextProps: Props) {
    return !RelayRow.compareProps(this.props, nextProps);
  }

  static compareProps(oldProps: Props, nextProps: Props) {
    return oldProps.hostname === nextProps.hostname && oldProps.selected === nextProps.selected;
  }

  render() {
    return (
      <Cell.CellButton
        onPress={this._handlePress}
        cellHoverStyle={this.props.selected ? styles.selected : undefined}
        style={[styles.base, this.props.selected ? styles.selected : undefined]}>
        <RelayStatusIndicator isActive={true} isSelected={this.props.selected} />

        <Cell.Label>{this.props.hostname}</Cell.Label>
      </Cell.CellButton>
    );
  }

  _handlePress = () => {
    if (this.props.onSelect) {
      this.props.onSelect();
    }
  };
}
