import * as React from 'react';
import styled from 'styled-components';
import { Component, Styles, View } from 'reactxp';
import { colors } from '../../config.json';
import * as Cell from './Cell';

const styles = {
  relayStatus: Styles.createViewStyle({
    width: 16,
    height: 16,
    borderRadius: 8,
    marginLeft: 4,
    marginRight: 12,
  }),
  inactive: Styles.createViewStyle({
    backgroundColor: colors.red95,
  }),
  active: Styles.createViewStyle({
    backgroundColor: colors.green90,
  }),
};

const TickIcon = styled(Cell.Icon)({
  marginRight: '8px',
});

interface IProps {
  active: boolean;
  selected: boolean;
}

export default class RelayStatusIndicator extends Component<IProps> {
  public render() {
    return this.props.selected ? (
      <TickIcon tintColor={colors.white} source="icon-tick" height={24} width={24} />
    ) : (
      <View style={[styles.relayStatus, this.props.active ? styles.active : styles.inactive]} />
    );
  }
}
