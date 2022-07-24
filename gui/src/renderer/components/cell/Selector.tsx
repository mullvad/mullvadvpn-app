import * as React from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { useBoolean } from '../../lib/utilityHooks';
import Accordion from '../Accordion';
import { AriaDetails, AriaInput, AriaLabel } from '../AriaGroup';
import ChevronButton from '../ChevronButton';
import { normalText } from '../common-styles';
import InfoButton from '../InfoButton';
import * as Cell from '.';

const StyledTitle = styled(Cell.Container)({
  display: 'flex',
  padding: 0,
});

const StyledTitleLabel = styled(Cell.SectionTitle)({
  flex: 1,
});

const StyledChevronButton = styled(ChevronButton)({
  padding: 0,
  marginRight: '16px',
});

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
  details?: React.ReactElement;
  expandable?: boolean;
  disabled?: boolean;
  thinTitle?: boolean;
}

export default function Selector<T>(props: ISelectorProps<T>) {
  const [expanded, , , toggleExpanded] = useBoolean(!props.expandable);

  const items = props.values.map((item, i) => {
    const selected = item.value === props.value;

    return (
      <SelectorCell
        key={i}
        value={item.value}
        selected={selected}
        disabled={props.disabled || item.disabled}
        forwardedRef={selected ? props.selectedCellRef : undefined}
        onSelect={props.onSelect}>
        {item.label}
      </SelectorCell>
    );
  });

  const title = props.title && (
    <StyledTitle>
      <AriaLabel>
        <StyledTitleLabel as="label" disabled={props.disabled} thin={props.thinTitle}>
          {props.title}
        </StyledTitleLabel>
      </AriaLabel>
      {props.details && (
        <AriaDetails>
          <InfoButton>{props.details}</InfoButton>
        </AriaDetails>
      )}
      {props.expandable && <StyledChevronButton up={expanded} onClick={toggleExpanded} />}
    </StyledTitle>
  );

  return (
    <AriaInput>
      <Cell.Section role="listbox" className={props.className}>
        {title}
        {props.expandable ? <Accordion expanded={expanded}>{items}</Accordion> : items}
      </Cell.Section>
    </AriaInput>
  );
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
