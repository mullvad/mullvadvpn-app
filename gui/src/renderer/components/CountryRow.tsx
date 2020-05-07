import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, Styles, View } from 'reactxp';
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
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
  children?: CityRowElement | CityRowElement[];
}

const styles = {
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 0,
  }),
  base: Styles.createViewStyle({
    paddingRight: 0,
    paddingLeft: 16,
  }),
};

export default class CountryRow extends Component<IProps> {
  private buttonRef = React.createRef<Cell.CellButton>();

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
          ref={this.buttonRef}
          style={styles.base}
          onPress={this.handlePress}
          disabled={!this.props.hasActiveRelays}
          selected={this.props.selected}>
          <RelayStatusIndicator
            active={this.props.hasActiveRelays}
            selected={this.props.selected}
          />
          <Cell.Label>{this.props.name}</Cell.Label>
          {hasChildren ? (
            <ChevronButton onClick={this.toggleCollapse} up={this.props.expanded} />
          ) : null}
        </Cell.CellButton>

        {hasChildren && (
          <Accordion
            expanded={this.props.expanded}
            onWillExpand={this.onWillExpand}
            onTransitionEnd={this.props.onTransitionEnd}
            animationDuration={150}>
            {this.props.children}
          </Accordion>
        )}
      </View>
    );
  }

  private toggleCollapse = (event: React.MouseEvent) => {
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

  private onWillExpand = (nextHeight: number) => {
    const buttonNode = ReactDOM.findDOMNode(this.buttonRef.current);
    if (buttonNode instanceof HTMLElement) {
      const buttonRect = buttonNode.getBoundingClientRect();
      this.props.onWillExpand?.(buttonRect, nextHeight);
    }
  };
}
