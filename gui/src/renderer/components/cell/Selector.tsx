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

  const isNonCustomItem = (value: T | U | undefined) =>
    props.items.some((item) => item.value === value) || props.automaticValue === value;

  const itemIsSelected = isNonCustomItem(value);

  // Value of custom input. The value is undefined when custom isn't picked.
  const [customValue, setCustomValue] = useState(itemIsSelected ? undefined : `${value}`);
  const customIsSelected = customValue !== undefined;

  const inputRef = useRef() as React.RefObject<HTMLInputElement>;

  const handleClickCustom = useCallback(() => {
    inputRef.current?.focus();
    setCustomValue((customValue) => customValue ?? '');
  }, [customValue, inputRef.current]);

  // This prevents the input blur event if another item is pressed. The blur event would cause a
  // submit/invalid which would immediately be followed by a new value which would make it change
  // to two different options one after the other.
  const handleMouseDown = useCallback((event: React.MouseEvent) => {
    if (event.target !== inputRef.current) {
      event.preventDefault();
    }
  }, []);

  const handleSelectValue = useCallback(
    (newValue: T | U | undefined) => {
      onSelect(newValue!);

      if (value === newValue) {
        inputRef.current?.blur();
      }
    },
    [value, onSelect],
  );

  const validateCustomValue = useCallback(
    (value: string) => validateValue?.(parseValue(value)) ?? true,
    [parseValue, validateValue],
  );

  const handleSubmitCustom = useCallback(
    (newStringValue: string) => {
      const newValue = parseValue(newStringValue);

      // If an already existing alternative is picked we want to switch to that one.
      if (isNonCustomItem(newValue) && newValue === value) {
        setCustomValue(undefined);
      }

      onSelect(newValue);
    },
    [value, parseValue, onSelect],
  );

  const handleInvalidCustom = useCallback(() => {
    setCustomValue(itemIsSelected ? undefined : `${value}`);
  }, [itemIsSelected, value]);

  // If a new value is received while custom is selected and that value is one of the alternatives,
  // then the custom alternative should be unselected.
  useEffect(() => {
    if (customIsSelected && itemIsSelected) {
      setCustomValue(undefined);
      inputRef.current?.blur();
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
          onClick={handleClickCustom}
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
              ref={inputRef}
              value={customValue ?? ''}
              placeholder={inputPlaceholder}
              inputMode={'numeric'}
              maxLength={maxLength ?? 4}
              onChangeValue={setCustomValue}
              onSubmitValue={handleSubmitCustom}
              onInvalidValue={handleInvalidCustom}
              submitOnBlur={true}
              validateValue={validateCustomValue}
              modifyValue={modifyValue}
            />
          </AriaInput>
        </StyledCustomContainer>
      </Selector>
    </div>
  );
}
