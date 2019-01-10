// @flow

import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import { Accordion } from '@mullvad/components';
import * as Cell from './Cell';
import CityRow from './CityRow';
import RelayStatusIndicator from './RelayStatusIndicator';
import ChevronButton from './ChevronButton';
import { colors } from '../../config';

type Props = {
  name: string,
  hasActiveRelays: boolean,
  selected: boolean,
  collapsed: boolean,
  onPress?: () => void,
  onCollapse?: (boolean) => void,
  children?: React.Element<typeof CityRow>,
};

const styles = {
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 0,
  }),
  base: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 20,
    paddingRight: 0,
  }),
  selected: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
};

export default class CountryRow extends Component<Props> {
  shouldComponentUpdate(nextProps: Props) {
    return !CountryRow.compareProps(this.props, nextProps);
  }

  static compareProps(oldProps: Props, nextProps: Props) {
    if (React.Children.count(oldProps.children) !== React.Children.count(nextProps.children)) {
      return false;
    }

    if (
      oldProps.name !== nextProps.name ||
      oldProps.hasActiveRelays !== nextProps.hasActiveRelays ||
      oldProps.selected !== nextProps.selected ||
      oldProps.collapsed !== nextProps.collapsed
    ) {
      return false;
    }

    const currChildren = React.Children.toArray(oldProps.children);
    const nextChildren = React.Children.toArray(nextProps.children);

    for (let i = 0; i < currChildren.length; i++) {
      const currChild = currChildren[i];
      const nextChild = nextChildren[i];

      if (!CityRow.compareProps(currChild.props, nextChild.props)) {
        return false;
      }
    }

    return true;
  }

  render() {
    const numChildren = React.Children.count(this.props.children);
    const onlyChild = numChildren === 1 ? this.props.children[0] : undefined;
    const numOnlyChildChildren = onlyChild ? React.Children.count(onlyChild.props.children) : 0;
    const hasChildren = numChildren > 1 || numOnlyChildChildren > 1;

    return (
      <View style={styles.container}>
        <Cell.CellButton
          cellHoverStyle={this.props.selected ? styles.selected : null}
          style={[styles.base, this.props.selected ? styles.selected : null]}
          onPress={this.props.onPress}
          disabled={!this.props.hasActiveRelays}
          testName="country">
          <RelayStatusIndicator
            isActive={this.props.hasActiveRelays}
            isSelected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>
          {hasChildren ? (
            <ChevronButton onPress={this._toggleCollapse} collapsed={this.props.collapsed} />
          ) : null}
        </Cell.CellButton>

        {hasChildren && (
          <Accordion collapsed={this.props.collapsed}>{this.props.children}</Accordion>
        )}
      </View>
    );
  }

  _toggleCollapse = (event: Event) => {
    if (this.props.onCollapse) {
      this.props.onCollapse(!this.props.collapsed);
    }
    event.stopPropagation();
  };
}
