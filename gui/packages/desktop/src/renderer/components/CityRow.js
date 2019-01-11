// @flow

import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import { Accordion } from '@mullvad/components';
import * as Cell from './Cell';
import RelayRow from './RelayRow';
import RelayStatusIndicator from './RelayStatusIndicator';
import ChevronButton from './ChevronButton';
import { colors } from '../../config';

type Props = {
  name: string,
  hasActiveRelays: boolean,
  selected: boolean,
  expanded: boolean,
  selected: boolean,
  onSelect?: () => void,
  onExpand?: () => void,
  children?: React.Element<typeof RelayRow>,
};

const styles = {
  base: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 40,
    backgroundColor: colors.blue40,
  }),
  selected: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
};

export default class CityRow extends Component<Props> {
  shouldComponentUpdate(nextProps: Props) {
    return !CityRow.compareProps(this.props, nextProps);
  }

  static compareProps(oldProps: Props, nextProps: Props) {
    if (React.Children.count(oldProps.children) !== React.Children.count(nextProps.children)) {
      return false;
    }

    if (
      oldProps.name !== nextProps.name ||
      oldProps.hasActiveRelays !== nextProps.hasActiveRelays ||
      oldProps.selected !== nextProps.selected ||
      oldProps.expanded !== nextProps.expanded
    ) {
      return false;
    }

    const currChildren = React.Children.toArray(oldProps.children);
    const nextChildren = React.Children.toArray(nextProps.children);

    for (let i = 0; i < currChildren.length; i++) {
      const currChild = currChildren[i];
      const nextChild = nextChildren[i];

      if (!RelayRow.compareProps(currChild.props, nextChild.props)) {
        return false;
      }
    }

    return true;
  }

  render() {
    const hasChildren = React.Children.count(this.props.children) > 1;

    return (
      <View>
        <Cell.CellButton
          onPress={this._handlePress}
          disabled={!this.props.hasActiveRelays}
          cellHoverStyle={this.props.selected ? styles.selected : null}
          style={[styles.base, this.props.selected ? styles.selected : null]}
          testName="city">
          <RelayStatusIndicator
            isActive={this.props.hasActiveRelays}
            isSelected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>

          {hasChildren && <ChevronButton onPress={this._toggleCollapse} up={this.props.expanded} />}
        </Cell.CellButton>

        {hasChildren && <Accordion expanded={this.props.expanded}>{this.props.children}</Accordion>}
      </View>
    );
  }

  _toggleCollapse = (event: Event) => {
    if (this.props.onExpand) {
      this.props.onExpand(!this.props.expanded);
    }
    event.stopPropagation();
  };

  _handlePress = () => {
    if (this.props.onSelect) {
      this.props.onSelect();
    }
  };
}
