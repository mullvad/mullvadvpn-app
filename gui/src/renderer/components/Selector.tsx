import * as React from 'react';
import { Component, Styles, Types } from 'reactxp';
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
  cell: {
    selectedHover: Styles.createButtonStyle({
      backgroundColor: colors.green,
    }),
  },
  invisibleIcon: Styles.createViewStyle({
    opacity: 0,
  }),
};

export default class Selector<T> extends Component<ISelectorProps<T>> {
  public render() {
    return (
      <Cell.Section style={[styles.section, this.props.style]}>
        {this.props.title && <Cell.SectionTitle>{this.props.title}</Cell.SectionTitle>}
        {this.props.values.map((item, i) => {
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
        })}
      </Cell.Section>
    );
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
      <Cell.CellButton
        style={this.props.selected ? styles.cell.selectedHover : undefined}
        cellHoverStyle={this.props.selected ? styles.cell.selectedHover : undefined}
        onPress={this.onPress}>
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
