import * as React from 'react';
import { Component, Styles } from 'reactxp';
import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import * as Cell from './Cell';
import RelayStatusIndicator from './RelayStatusIndicator';

interface IProps {
  location: RelayLocation;
  active: boolean;
  hostname: string;
  selected: boolean;
  onSelect?: (location: RelayLocation) => void;
}

const styles = {
  base: Styles.createViewStyle({
    paddingRight: 0,
    paddingLeft: 48,
    backgroundColor: colors.blue20,
  }),
};

export default class RelayRow extends Component<IProps> {
  public static compareProps(oldProps: IProps, nextProps: IProps) {
    return (
      oldProps.hostname === nextProps.hostname &&
      oldProps.selected === nextProps.selected &&
      oldProps.active === nextProps.active &&
      compareRelayLocation(oldProps.location, nextProps.location)
    );
  }

  public shouldComponentUpdate(nextProps: IProps) {
    return !RelayRow.compareProps(this.props, nextProps);
  }

  public render() {
    return (
      <Cell.CellButton
        onPress={this.handlePress}
        selected={this.props.selected}
        disabled={!this.props.active}
        style={styles.base}>
        <RelayStatusIndicator isActive={this.props.active} isSelected={this.props.selected} />

        <Cell.Label>{this.props.hostname}</Cell.Label>
      </Cell.CellButton>
    );
  }

  private handlePress = () => {
    if (this.props.onSelect) {
      this.props.onSelect(this.props.location);
    }
  };
}
