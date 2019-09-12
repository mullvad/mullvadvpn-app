import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import { colors } from '../../config.json';
import * as Cell from './Cell';

const styles = {
  relayStatus: Styles.createViewStyle({
    width: 16,
    height: 16,
    borderRadius: 8,
    marginLeft: 12,
    marginRight: 4,
  }),
  inactive: Styles.createViewStyle({
    backgroundColor: colors.red95,
  }),
  active: Styles.createViewStyle({
    backgroundColor: colors.green90,
  }),
};

interface IProps {
  isActive: boolean;
  isSelected: boolean;
}

export default class RelayStatusIndicator extends Component<IProps> {
  public render() {
    return this.props.isSelected ? (
      <Cell.Icon tintColor={colors.white} source="icon-tick" height={24} width={24} />
    ) : (
      <View style={[styles.relayStatus, this.props.isActive ? styles.active : styles.inactive]} />
    );
  }
}
