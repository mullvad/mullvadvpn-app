import { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { messages } from '../../../shared/gettext';
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

export interface SelectorItem<T> {
  label: string;
  value: T;
  disabled?: boolean;
}

interface SelectorProps<T, U> {
  title?: string;
  items: Array<SelectorItem<T>>;
  value: T | U;
  onSelect: (value: T | U) => void;
  selectedCellRef?: React.Ref<HTMLElement>;
  className?: string;
  details?: React.ReactElement;
  expandable?: boolean;
  disabled?: boolean;
  thinTitle?: boolean;
  automaticLabel?: string;
  automaticValue?: U;
}

export default function Selector<T, U>(props: SelectorProps<T, U>) {
  const [expanded, , , toggleExpanded] = useBoolean(!props.expandable);

  const items = props.items.map((item) => {
    const selected = props.value === item.value;
    const ref = selected ? (props.selectedCellRef as React.Ref<HTMLButtonElement>) : undefined;

    return (
      <SelectorCell
        key={`value-${item.value}`}
        value={item.value}
        isSelected={selected}
        disabled={props.disabled || item.disabled}
        forwardedRef={ref}
        onSelect={props.onSelect}>
        {item.label}
      </SelectorCell>
    );
  });

  if (props.automaticValue !== undefined) {
    const selected = props.value === props.automaticValue;
    const ref = selected ? (props.selectedCellRef as React.Ref<HTMLButtonElement>) : undefined;

    items.unshift(
      <SelectorCell
        key={'automatic'}
        value={props.automaticValue}
        isSelected={selected}
        disabled={props.disabled}
        forwardedRef={ref}
        onSelect={props.onSelect}>
        {props.automaticLabel ?? messages.gettext('Automatic')}
      </SelectorCell>,
    );
  }

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

interface SelectorCellProps<T> {
  value: T;
  isSelected: boolean;
  disabled?: boolean;
  onSelect: (value: T) => void;
  children: React.ReactNode | Array<React.ReactNode>;
  forwardedRef?: React.Ref<HTMLButtonElement>;
}

function SelectorCell<T>(props: SelectorCellProps<T>) {
  const handleClick = useCallback(() => {
    if (!props.isSelected) {
      props.onSelect(props.value);
    }
  }, [props.isSelected, props.onSelect, props.value]);

  return (
    <Cell.CellButton
      ref={props.forwardedRef}
      onClick={handleClick}
      selected={props.isSelected}
      disabled={props.disabled}
      role="option"
      aria-selected={props.isSelected}
      aria-disabled={props.disabled}>
      <StyledCellIcon
        visible={props.isSelected}
        source="icon-tick"
        width={18}
        tintColor={colors.white}
      />
      <StyledLabel>{props.children}</StyledLabel>
    </Cell.CellButton>
  );
}
