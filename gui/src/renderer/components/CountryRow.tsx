import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import Accordion from './Accordion';
import * as Cell from './Cell';
import ChevronButton from './ChevronButton';
import CityRow from './CityRow';
import RelayStatusIndicator from './RelayStatusIndicator';

type CityRowElement = React.ReactElement<CityRow['props']>;

interface IProps {
  name: string;
  hasActiveRelays: boolean;
  location: RelayLocation;
  selected: boolean;
  expanded: boolean;
  onSelect?: (location: RelayLocation) => void;
  onExpand?: (location: RelayLocation, value: boolean) => void;
  children?: CityRowElement | CityRowElement[];
}

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

export default class CountryRow extends Component<IProps> {
  public static compareProps(oldProps: IProps, nextProps: IProps) {
    if (React.Children.count(oldProps.children) !== React.Children.count(nextProps.children)) {
      return false;
    }

    if (
      oldProps.name !== nextProps.name ||
      oldProps.hasActiveRelays !== nextProps.hasActiveRelays ||
      oldProps.selected !== nextProps.selected ||
      oldProps.expanded !== nextProps.expanded ||
      !compareRelayLocation(oldProps.location, nextProps.location)
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

  public shouldComponentUpdate(nextProps: IProps) {
    return !CountryRow.compareProps(this.props, nextProps);
  }

  public render() {
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
          onPress={this.handlePress}
          disabled={!this.props.hasActiveRelays}>
          <RelayStatusIndicator
            isActive={this.props.hasActiveRelays}
            isSelected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>
          {hasChildren ? (
            <ChevronButton onPress={this.toggleCollapse} up={this.props.expanded} />
          ) : null}
        </Cell.CellButton>

        {hasChildren && <Accordion expanded={this.props.expanded}>{this.props.children}</Accordion>}
      </View>
    );
  }

  private toggleCollapse = (event: Types.SyntheticEvent) => {
    if (this.props.onExpand) {
      this.props.onExpand(this.props.location, !this.props.expanded);
    }
    event.stopPropagation();
  };

  private handlePress = () => {
    if (this.props.onSelect) {
      this.props.onSelect(this.props.location);
    }
  };
}
