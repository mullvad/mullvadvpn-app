import * as React from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { AriaInput, AriaLabel } from '../AriaGroup';
import { normalText } from '../common-styles';
import * as Cell from '.';

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
  selectedCellRef?: React.Ref<HTMLButtonElement>;
  className?: string;
}

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
          forwardedRef={selected ? this.props.selectedCellRef : undefined}
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
        <Cell.Section role="listbox" className={this.props.className}>
          {title}
          {items}
        </Cell.Section>
      </AriaInput>
    );
  }
}

const StyledCellIcon = styled(Cell.Icon)((props: { visible: boolean }) => ({
  opacity: props.visible ? 1 : 0,
  marginRight: '8px',
}));

const StyledLabel = styled(Cell.Label)(normalText, {
  fontWeight: 400,
});

interface ISelectorCellProps<T> {
  value: T;
  selected: boolean;
  disabled?: boolean;
  onSelect: (value: T) => void;
  children?: React.ReactText;
  forwardedRef?: React.Ref<HTMLButtonElement>;
}

class SelectorCell<T> extends React.Component<ISelectorCellProps<T>> {
  public render() {
    return (
      <Cell.CellButton
        ref={this.props.forwardedRef}
        onClick={this.onClick}
        selected={this.props.selected}
        disabled={this.props.disabled}
        role="option"
        aria-selected={this.props.selected}
        aria-disabled={this.props.disabled}>
        <StyledCellIcon
          visible={this.props.selected}
          source="icon-tick"
          width={18}
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
