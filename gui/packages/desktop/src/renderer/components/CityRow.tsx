import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import Accordion from './Accordion';
import * as Cell from './Cell';
import ChevronButton from './ChevronButton';
import RelayRow from './RelayRow';
import RelayStatusIndicator from './RelayStatusIndicator';

type RelayRowElement = React.ReactElement<RelayRow['props']>;

interface IProps {
  name: string;
  hasActiveRelays: boolean;
  location: RelayLocation;
  selected: boolean;
  expanded: boolean;
  onSelect?: (location: RelayLocation) => void;
  onExpand?: (location: RelayLocation, value: boolean) => void;
  children?: RelayRowElement | RelayRowElement[];
}

const styles = {
  base: Styles.createButtonStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingRight: 0,
    paddingLeft: 40,
    backgroundColor: colors.blue40,
  }),
  selected: Styles.createButtonStyle({
    backgroundColor: colors.green,
  }),
};

export default class CityRow extends Component<IProps> {
  public static compareProps(oldProps: IProps, nextProps: IProps): boolean {
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

    const currChildren = React.Children.toArray(oldProps.children || []) as RelayRowElement[];
    const nextChildren = React.Children.toArray(nextProps.children || []) as RelayRowElement[];

    for (let i = 0; i < currChildren.length; i++) {
      const currChild = currChildren[i];
      const nextChild = nextChildren[i];

      if (!RelayRow.compareProps(currChild.props, nextChild.props)) {
        return false;
      }
    }

    return true;
  }

  public shouldComponentUpdate(nextProps: IProps) {
    return !CityRow.compareProps(this.props, nextProps);
  }

  public render() {
    const hasChildren = React.Children.count(this.props.children) > 1;

    return (
      <View>
        <Cell.CellButton
          onPress={this.handlePress}
          disabled={!this.props.hasActiveRelays}
          cellHoverStyle={this.props.selected ? styles.selected : undefined}
          style={[styles.base, this.props.selected ? styles.selected : undefined]}>
          <RelayStatusIndicator
            isActive={this.props.hasActiveRelays}
            isSelected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>

          {hasChildren && <ChevronButton onPress={this.toggleCollapse} up={this.props.expanded} />}
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
