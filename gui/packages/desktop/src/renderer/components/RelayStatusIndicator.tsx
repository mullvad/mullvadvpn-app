import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import * as Cell from './Cell';
import { colors } from '../../config.json';

const styles = {
  relay_status: Styles.createViewStyle({
    width: 16,
    height: 16,
    borderRadius: 8,
    marginLeft: 4,
    marginRight: 4,
  }),
  relay_status__inactive: Styles.createViewStyle({
    backgroundColor: colors.red95,
  }),
  relay_status__active: Styles.createViewStyle({
    backgroundColor: colors.green90,
  }),
  tick_icon: Styles.createViewStyle({
    marginLeft: 0,
    marginRight: 0,
  }),
};

type Props = {
  isActive: boolean;
  isSelected: boolean;
};

export default class RelayStatusIndicator extends Component<Props> {
  render() {
    return this.props.isSelected ? (
      <Cell.Icon
        style={styles.tick_icon}
        tintColor={colors.white}
        source="icon-tick"
        height={24}
        width={24}
      />
    ) : (
      <View
        style={[
          styles.relay_status,
          this.props.isActive ? styles.relay_status__active : styles.relay_status__inactive,
        ]}
      />
    );
  }
}
