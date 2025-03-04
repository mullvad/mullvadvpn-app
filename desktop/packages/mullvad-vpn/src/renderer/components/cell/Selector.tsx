import { useCallback, useRef, useState } from 'react';
import styled from 'styled-components';

import { messages } from '../../../shared/gettext';
import { Icon } from '../../lib/components';
import { Colors, Spacings } from '../../lib/foundations';
import { useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';
import { useStyledRef } from '../../lib/utility-hooks';
import { AriaDetails, AriaInput, AriaLabel } from '../AriaGroup';
import InfoButton from '../InfoButton';
import * as Cell from '.';

const StyledTitleLabel = styled(Cell.SectionTitle)({
  flex: 1,
});

const StyledInfoButton = styled(InfoButton)({
  marginRight: Spacings.medium,
});

export interface SelectorItem<T> {
  label: string;
  value: T;
  disabled?: boolean;
  'data-testid'?: string;
  details?: { path: RoutePath; ariaLabel: string };
  subLabel?: string;
}

// T represents the available values and U represent the value of "Automatic"/"Any" if there is one.
interface CommonSelectorProps<T, U> {
  title?: string;
  items: Array<SelectorItem<T>>;
  value: T | U;
  selectedCellRef?: React.Ref<HTMLElement>;
  className?: string;
  infoTitle?: string;
  details?: React.ReactElement;
  expandable?: { expandable: boolean; id: string };
  disabled?: boolean;
  thinTitle?: boolean;
  automaticLabel?: string;
  automaticValue?: U;
  automaticTestId?: string;
  children?: React.ReactNode | Array<React.ReactNode>;
}

interface SelectorProps<T, U> extends CommonSelectorProps<T, U> {
  onSelect: (value: T | U) => void;
}

export default function Selector<T, U>(props: SelectorProps<T, U>) {
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
        onSelect={props.onSelect}
        subLabel={item.subLabel}
        details={item.details}
        data-testid={item['data-testid']}>
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
        data-testid={props.automaticTestId}
        value={props.automaticValue}
        isSelected={selected}
        disabled={props.disabled}
        forwardedRef={ref}
        onSelect={props.onSelect}>
        {props.automaticLabel ?? messages.gettext('Automatic')}
      </SelectorCell>,
    );
  }

  const title = props.title ? (
    <>
      <AriaLabel>
        <StyledTitleLabel as="label" disabled={props.disabled} $thin={props.thinTitle}>
          {props.title}
        </StyledTitleLabel>
      </AriaLabel>
      {props.details && (
        <AriaDetails>
          <StyledInfoButton title={props.infoTitle}>{props.details}</StyledInfoButton>
        </AriaDetails>
      )}
    </>
  ) : undefined;

  // Add potential additional items to the list. Used for custom entry.
  const children = (
    <Cell.Group $noMarginBottom>
      {items}
      {props.children}
    </Cell.Group>
  );

  if (props.expandable?.expandable) {
    return (
      <AriaInput>
        <Cell.ExpandableSection
          role="listbox"
          expandedInitially={false}
          className={props.className}
          sectionTitle={title}
          expandableId={props.expandable.id}>
          {children}
        </Cell.ExpandableSection>
      </AriaInput>
    );
  } else {
    return (
      <AriaInput>
        <Cell.Section role="listbox" className={props.className} sectionTitle={title}>
          {children}
        </Cell.Section>
      </AriaInput>
    );
  }
}

const StyledCellIcon = styled(Icon)<{ $visible: boolean }>((props) => ({
  opacity: props.$visible ? 1 : 0,
  marginRight: '8px',
}));

interface SelectorCellProps<T> {
  value: T;
  isSelected: boolean;
  disabled?: boolean;
  onSelect: (value: T) => void;
  children: string;
  subLabel?: string;
  forwardedRef?: React.Ref<HTMLButtonElement>;
  'data-testid'?: string;
  details?: SelectorItem<unknown>['details'];
}

const StyledSelectorCell = styled.div({
  display: 'flex',
});

const StyledSideButton = styled(Cell.SideButton)({
  marginBottom: '1px',
});

function SelectorCell<T>(props: SelectorCellProps<T>) {
  const { onSelect } = props;

  const { push } = useHistory();

  const handleClick = useCallback(() => {
    if (!props.isSelected) {
      onSelect(props.value);
    }
  }, [props.isSelected, onSelect, props.value]);

  const navigate = useCallback(() => {
    if (props.details) {
      push(props.details.path);
    }
  }, [props.details, push]);

  return (
    <StyledSelectorCell>
      <Cell.CellButton
        ref={props.forwardedRef}
        onClick={handleClick}
        selected={props.isSelected}
        disabled={props.disabled}
        role="option"
        aria-selected={props.isSelected}
        aria-disabled={props.disabled}
        data-testid={props['data-testid']}>
        <StyledCellIcon $visible={props.isSelected} icon="checkmark" />
        <SelectorCellLabel subLabel={props.subLabel}>{props.children}</SelectorCellLabel>
      </Cell.CellButton>
      {props.details && (
        <StyledSideButton
          $backgroundColor={Colors.blue40}
          $backgroundColorHover={Colors.blue80}
          aria-label={props.details.ariaLabel}
          onClick={navigate}>
          <Icon icon="chevron-right" />
        </StyledSideButton>
      )}
    </StyledSelectorCell>
  );
}

interface SelectorCellLabelProps {
  children: string;
  subLabel?: string;
}

function SelectorCellLabel(props: SelectorCellLabelProps) {
  if (props.subLabel) {
    return (
      <Cell.LabelContainer>
        <Cell.ValueLabel>{props.children}</Cell.ValueLabel>
        {props.subLabel && <Cell.SubLabel>{props.subLabel}</Cell.SubLabel>}
      </Cell.LabelContainer>
    );
  } else {
    return <Cell.ValueLabel>{props.children}</Cell.ValueLabel>;
  }
}

interface StyledCustomContainerProps {
  selected: boolean;
}

const StyledCustomContainer = styled(Cell.Container)<StyledCustomContainerProps>((props) => ({
  backgroundColor: props.selected ? Colors.green : Colors.blue40,
  '&&:hover': {
    backgroundColor: props.selected ? Colors.green : Colors.blue,
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
    value: _value,
    inputPlaceholder,
    onSelect,
    maxLength,
    selectedCellRef,
    validateValue,
    parseValue,
    modifyValue,
    ...otherProps
  } = props;

  const [value, setValue] = useState(props.value);
  // Disables submitting of custom input when another item has been pressed.
  const allowSubmitCustom = useRef(false);

  const isNonCustomItem = useCallback(
    (value: T | U | undefined) =>
      props.items.some((item) => item.value === value) || props.automaticValue === value,
    [props.automaticValue, props.items],
  );

  const itemIsSelected = isNonCustomItem(value);
  // Value of custom input. The value is undefined when custom isn't picked.
  const [customValue, setCustomValue] = useState(itemIsSelected ? undefined : `${value}`);
  const customIsSelected = customValue !== undefined;

  const inputRef = useStyledRef<HTMLInputElement>();

  const handleClickCustom = useCallback(() => {
    inputRef.current?.focus();
    // After focusing the input it should be allowed to submit custom values.
    allowSubmitCustom.current = true;
    setCustomValue((customValue) => customValue ?? '');
  }, [inputRef]);

  const handleSelectItem = useCallback(
    (newValue: T | U | undefined) => {
      setCustomValue(undefined);
      setValue(newValue);
      // When pressing an item the blur shouldn't be triggered since that would cause the input
      // value to be propagated as the new value.
      allowSubmitCustom.current = false;
      inputRef.current?.blur();

      onSelect(newValue!);
    },
    [inputRef, onSelect],
  );

  const validateCustomValue = useCallback(
    (value: string) => validateValue?.(parseValue(value)) ?? true,
    [parseValue, validateValue],
  );

  const handleSubmitCustom = useCallback(
    (newStringValue: string) => {
      if (allowSubmitCustom.current) {
        const newValue = parseValue(newStringValue);

        if (isNonCustomItem(newValue)) {
          handleSelectItem(newValue);
        } else {
          setValue(newValue);
          onSelect(newValue);
        }
      }
    },
    [parseValue, isNonCustomItem, handleSelectItem, onSelect],
  );

  const handleInvalidCustom = useCallback(
    () => setCustomValue(itemIsSelected ? undefined : `${value}`),
    [itemIsSelected, value],
  );

  // Delay blur event until onMouseUp resulting in handleSelectItem being called before
  // handleSubmitCustomValue and handleInvalidCustom. Clicking on the input should still move the
  // cursor and therefore needs to be an exception to this.
  const handleMouseDown = useCallback(
    (event: React.MouseEvent) => {
      if (event.target !== inputRef.current) {
        event.preventDefault();
      }
    },
    [inputRef],
  );

  return (
    <div onMouseDown={handleMouseDown}>
      <Selector<T | undefined, U>
        {...otherProps}
        onSelect={handleSelectItem}
        value={customIsSelected ? undefined : value}>
        <StyledCustomContainer
          ref={customIsSelected ? props.selectedCellRef : undefined}
          onClick={handleClickCustom}
          selected={customIsSelected}
          disabled={props.disabled}
          role="option"
          aria-selected={customIsSelected}
          aria-disabled={props.disabled}>
          <StyledCellIcon $visible={customIsSelected} icon="checkmark" />
          <Cell.ValueLabel>{messages.gettext('Custom')}</Cell.ValueLabel>
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
