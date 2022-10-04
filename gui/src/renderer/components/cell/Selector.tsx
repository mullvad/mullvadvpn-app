import { useCallback, useEffect, useRef, useState } from 'react';
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

// T represents the available values and U represent the value of "Automatic"/"Any" if there is one.
interface CommonSelectorProps<T, U> {
  title?: string;
  items: Array<SelectorItem<T>>;
  value: T | U;
  selectedCellRef?: React.Ref<HTMLElement>;
  className?: string;
  details?: React.ReactElement;
  expandable?: boolean;
  disabled?: boolean;
  thinTitle?: boolean;
  automaticLabel?: string;
  automaticValue?: U;
  children?: React.ReactNode | Array<React.ReactNode>;
}

interface SelectorProps<T, U> extends CommonSelectorProps<T, U> {
  onSelect: (value: T | U) => void;
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

  // Add potential additional items to the list. Used for custom entry.
  const children = (
    <Cell.Group noMarginBottom>
      {items}
      {props.children}
    </Cell.Group>
  );

  return (
    <AriaInput>
      <Cell.Section role="listbox" className={props.className}>
        {title}
        {props.expandable ? <Accordion expanded={expanded}>{children}</Accordion> : children}
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

interface StyledCustomContainerProps {
  selected: boolean;
}

const StyledCustomContainer = styled(Cell.Container)((props: StyledCustomContainerProps) => ({
  backgroundColor: props.selected ? colors.green : colors.blue40,
  ':hover': {
    backgroundColor: props.selected ? colors.green : colors.blue,
  },
}));

// Adding undefined as possible value of the selector to be able to select nothing.
interface SelectorWithCustomItemProps<T, U> extends CommonSelectorProps<T | undefined, U> {
  inputPlaceholder: string;
  onSelect: (value: T | U) => void;
  parseValue: (value: string) => T;
  validateValue?: (value: T) => boolean;
  maxLength?: number;
  selectedCellRef?: React.Ref<HTMLDivElement>;
  modifyValue?: (value: string) => string;
}

export function SelectorWithCustomItem<T, U>(props: SelectorWithCustomItemProps<T, U>) {
  const {
    value,
    inputPlaceholder,
    onSelect,
    maxLength,
    selectedCellRef,
    validateValue,
    parseValue,
    modifyValue,
    ...otherProps
  } = props;

  // The component needs to keep track of when the custom item should look selected before it has a
  // value.
  const [customWithoutValue, setCustomWithoutValue, unsetCustomWithoutValue] = useBoolean(false);

  const isNonCustomItem = (value: T | U | undefined) =>
    props.items.some((item) => item.value === value) || props.automaticValue === value;

  const itemIsSelected = isNonCustomItem(value);
  const customIsSelected = !itemIsSelected || customWithoutValue;

  // The input key is used to clear the input state.
  const [inputKey, setInputKey] = useState(1);
  const resetInput = () => setInputKey((key) => key + 1);
  const inputRef = useRef() as React.RefObject<HTMLInputElement>;

  const handleClick = useCallback(() => {
    inputRef.current?.focus();
    if (!customIsSelected) {
      setCustomWithoutValue();
    }
  }, [customIsSelected, inputRef.current]);

  const handleMouseDown = useCallback((event: React.MouseEvent) => event.preventDefault(), []);

  // Wrap onSelect to be able to catch when a new value is selected during the
  // customIsSelectedWithoutValue phase. Value wont be undefined here since undefined items aren't
  // allowed.
  const handleSelectValue = useCallback(
    (newValue: T | U | undefined) => {
      resetInput();
      onSelect(newValue!);
    },
    [value, onSelect],
  );

  const validateStringValue = useCallback(
    (value: string) => validateValue?.(parseValue(value)) ?? true,
    [parseValue, validateValue],
  );

  const handleSubmit = useCallback(
    (stringValue: string) => {
      const value = parseValue(stringValue);

      if (isNonCustomItem(value)) {
        resetInput();
      }

      onSelect(value);
    },
    [parseValue, onSelect],
  );

  const handleInvalid = useCallback(() => {
    resetInput();
    unsetCustomWithoutValue();
  }, []);

  useEffect(() => {
    if (customWithoutValue && itemIsSelected) {
      unsetCustomWithoutValue();
    }
  }, [value]);

  return (
    <div onMouseDown={handleMouseDown}>
      <Selector<T | undefined, U>
        {...otherProps}
        onSelect={handleSelectValue}
        value={customIsSelected ? undefined : value}>
        <StyledCustomContainer
          ref={customIsSelected ? props.selectedCellRef : undefined}
          onClick={handleClick}
          selected={customIsSelected}
          disabled={props.disabled}
          role="option"
          aria-selected={customIsSelected}
          aria-disabled={props.disabled}>
          <StyledCellIcon
            visible={customIsSelected}
            source="icon-tick"
            width={18}
            tintColor={colors.white}
          />
          <StyledLabel>{messages.gettext('Custom')}</StyledLabel>
          <AriaInput>
            <Cell.AutoSizingTextInput
              key={inputKey}
              ref={inputRef}
              value={itemIsSelected || customWithoutValue ? '' : `${props.value}`}
              placeholder={inputPlaceholder}
              inputMode={'numeric'}
              maxLength={maxLength ?? 4}
              onSubmitValue={handleSubmit}
              onInvalidValue={handleInvalid}
              submitOnBlur={true}
              validateValue={validateStringValue}
              modifyValue={modifyValue}
            />
          </AriaInput>
        </StyledCustomContainer>
      </Selector>
    </div>
  );
}
