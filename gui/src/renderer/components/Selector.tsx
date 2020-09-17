import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { AriaInput, AriaLabel } from './AriaGroup';
import * as Cell from './Cell';

export interface ISelectorItem<T> {
  label: string;
  value: T;
  disabled?: boolean;
}

interface ISelectorProps<T> {
  title?: string;
  values: Array<ISelectorItem<T>>;
  value: T;
  onSelect: (value: T) => void;
  selectedCellRef?: React.Ref<SelectorCell<T>>;
  className?: string;
}

const Section = styled(Cell.Section)({
  marginBottom: 20,
});

export default class Selector<T> extends React.Component<ISelectorProps<T>> {
  public render() {
    const items = this.props.values.map((item, i) => {
      const selected = item.value === this.props.value;

      return (
        <SelectorCell
          key={i}
          value={item.value}
          selected={selected}
          disabled={item.disabled}
          ref={selected ? this.props.selectedCellRef : undefined}
          onSelect={this.props.onSelect}>
          {item.label}
        </SelectorCell>
      );
    });

    const title = this.props.title && (
      <AriaLabel>
        <Cell.SectionTitle as="label">{this.props.title}</Cell.SectionTitle>
      </AriaLabel>
    );

    return (
      <AriaInput>
        <Section role="listbox" className={this.props.className}>
          {title}
          {items}
        </Section>
      </AriaInput>
    );
  }
}

const StyledCellIcon = styled(Cell.Icon)((props: { visible: boolean }) => ({
  opacity: props.visible ? 1 : 0,
  marginRight: '8px',
}));

const StyledLabel = styled(Cell.Label)({
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
});

interface ISelectorCellProps<T> {
  value: T;
  selected: boolean;
  disabled?: boolean;
  onSelect: (value: T) => void;
  children?: React.ReactText;
}

export class SelectorCell<T> extends React.Component<ISelectorCellProps<T>> {
  public render() {
    return (
      <Cell.CellButton
        onClick={this.onClick}
        selected={this.props.selected}
        disabled={this.props.disabled}
        role="option"
        aria-selected={this.props.selected}
        aria-disabled={this.props.disabled}>
        <StyledCellIcon
          visible={this.props.selected}
          source="icon-tick"
          width={24}
          height={24}
          tintColor={colors.white}
        />
        <StyledLabel>{this.props.children}</StyledLabel>
      </Cell.CellButton>
    );
  }

  private onClick = () => {
    if (!this.props.selected) {
      this.props.onSelect(this.props.value);
    }
  };
}
