import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import * as Cell from './Cell';

export interface ISelectorItem<T> {
  label: string;
  value: T;
}

interface ISelectorProps<T> {
  style?: Types.ViewStyleRuleSet;
  title?: string;
  values: Array<ISelectorItem<T>>;
  value: T;
  onSelect: (value: T) => void;
  selectedCellRef?: React.Ref<SelectorCell<T>>;
}

const styles = {
  section: Styles.createViewStyle({
    marginBottom: 24,
  }),
  invisibleIcon: Styles.createViewStyle({
    opacity: 0,
  }),
};

export default class Selector<T> extends Component<ISelectorProps<T>> {
  public render() {
    const items = this.props.values.map((item, i) => {
      const selected = item.value === this.props.value;

      return (
        <SelectorCell
          key={i}
          value={item.value}
          selected={selected}
          ref={selected ? this.props.selectedCellRef : undefined}
          onSelect={this.props.onSelect}>
          {item.label}
        </SelectorCell>
      );
    });

    if (this.props.title) {
      return <Cell.Section style={[styles.section, this.props.style]}>{items}</Cell.Section>;
    } else {
      return <View style={[styles.section, this.props.style]}>{items}</View>;
    }
  }
}

interface ISelectorCellProps<T> {
  value: T;
  selected: boolean;
  onSelect: (value: T) => void;
  children?: React.ReactText;
}

export class SelectorCell<T> extends Component<ISelectorCellProps<T>> {
  public render() {
    return (
      <Cell.CellButton onPress={this.onPress} selected={this.props.selected}>
        <Cell.Icon
          style={this.props.selected ? undefined : styles.invisibleIcon}
          source="icon-tick"
          width={24}
          height={24}
          tintColor={colors.white}
        />
        <Cell.Label>{this.props.children}</Cell.Label>
      </Cell.CellButton>
    );
  }

  private onPress = () => {
    if (!this.props.selected) {
      this.props.onSelect(this.props.value);
    }
  };
}
