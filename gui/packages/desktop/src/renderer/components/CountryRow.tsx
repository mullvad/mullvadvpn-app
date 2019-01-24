import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import { Accordion } from '@mullvad/components';
import * as Cell from './Cell';
import CityRow from './CityRow';
import RelayStatusIndicator from './RelayStatusIndicator';
import ChevronButton from './ChevronButton';
import { colors } from '../../config.json';

type CityRowElement = React.ReactElement<CityRow['props']>;

type Props = {
  name: string;
  hasActiveRelays: boolean;
  selected: boolean;
  expanded: boolean;
  onSelect?: () => void;
  onExpand?: (value: boolean) => void;
  children?: CityRowElement | CityRowElement[];
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
      oldProps.expanded !== nextProps.expanded
    ) {
      return false;
    }

    const currChildren = React.Children.toArray(oldProps.children || []) as CityRowElement[];
    const nextChildren = React.Children.toArray(nextProps.children || []) as CityRowElement[];

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
    const childrenArray = React.Children.toArray(this.props.children || []) as CityRowElement[];
    const numChildren = childrenArray.length;
    const onlyChild = numChildren === 1 ? childrenArray[0] : undefined;
    const numOnlyChildChildren = onlyChild
      ? React.Children.count(onlyChild.props.children || [])
      : 0;
    const hasChildren = numChildren > 1 || numOnlyChildChildren > 1;

    return (
      <View style={styles.container}>
        <Cell.CellButton
          cellHoverStyle={this.props.selected ? styles.selected : undefined}
          style={[styles.base, this.props.selected ? styles.selected : undefined]}
          onPress={this._handlePress}
          disabled={!this.props.hasActiveRelays}>
          <RelayStatusIndicator
            isActive={this.props.hasActiveRelays}
            isSelected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>
          {hasChildren ? (
            <ChevronButton onPress={this._toggleCollapse} up={this.props.expanded} />
          ) : null}
        </Cell.CellButton>

        {hasChildren && <Accordion expanded={this.props.expanded}>{this.props.children}</Accordion>}
      </View>
    );
  }

  _toggleCollapse = (event: Types.SyntheticEvent) => {
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
