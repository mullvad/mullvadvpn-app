import * as React from 'react';
import { Component, Styles } from 'reactxp';
import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import * as Cell from './Cell';
import RelayStatusIndicator from './RelayStatusIndicator';

interface IProps {
  location: RelayLocation;
  hostname: string;
  selected: boolean;
  onSelect?: (location: RelayLocation) => void;
}

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

export default class RelayRow extends Component<IProps> {
  public static compareProps(oldProps: IProps, nextProps: IProps) {
    return (
      oldProps.hostname === nextProps.hostname &&
      oldProps.selected === nextProps.selected &&
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
        cellHoverStyle={this.props.selected ? styles.selected : undefined}
        style={[styles.base, this.props.selected ? styles.selected : undefined]}>
        <RelayStatusIndicator isActive={true} isSelected={this.props.selected} />

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
