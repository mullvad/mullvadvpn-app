import { useCallback, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { useScheduler } from '../../../shared/scheduler';
import { useBoolean } from '../../lib/utilityHooks';
import { AriaInput } from '../AriaGroup';
import { smallNormalText } from '../common-styles';
import CustomScrollbars from '../CustomScrollbars';
import ImageView from '../ImageView';

export interface SettingsSelectItem<T extends string> {
  value: T;
  label: string;
}

const StyledSelect = styled.div.attrs({ tabIndex: 0 })(smallNormalText, {
  display: 'flex',
  flex: 1,
  position: 'relative',
  background: 'transparent',
  border: 'none',
  color: colors.white,
  borderRadius: '4px',
  height: '26px',

  '&&:focus': {
    outline: `1px ${colors.darkBlue} solid`,
    backgroundColor: colors.blue,
  },
});

const StyledItems = styled.div<{ $direction: 'down' | 'up' }>((props) => ({
  display: 'flex',
  flexDirection: 'column',
  position: 'absolute',
  top: props.$direction === 'down' ? 'calc(100% + 4px)' : 'auto',
  bottom: props.$direction === 'up' ? 'calc(100% + 4px)' : 'auto',
  right: '-1px',
  backgroundColor: colors.darkBlue,
  border: `1px ${colors.darkerBlue} solid`,
  borderRadius: '4px',
  padding: '4px 8px',
  maxHeight: '250px',
  overflowY: 'hidden',
  zIndex: 2,
}));

const StyledSelectedContainer = styled.div({
  overflow: 'hidden',
  width: 'fit-content',
  maxWidth: '170px',
});

const StyledSelectedContainerInner = styled.div({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'end',
  height: '100%',
});

const StyledSelectedText = styled.span({
  display: 'inline-block',
  maxWidth: 'calc(100% - 30px)',
  marginLeft: '12px',
  whiteSpace: 'nowrap',
  textOverflow: 'ellipsis',
  overflow: 'hidden',
});

const StyledInvisibleItems = styled.div({
  padding: '0 29px 31px',
  visibility: 'hidden',
});

const StyledInvisibleItemsInner = styled.div({
  whiteSpace: 'nowrap',
});

const StyledChevron = styled(ImageView)({
  marginLeft: '6px',
  marginRight: '5px',
});

interface SettingsSelectProps<T extends string> {
  defaultValue?: T;
  items: Array<SettingsSelectItem<T>>;
  onUpdate: (value: T) => void;
  direction?: 'down' | 'up';
  // eslint-disable-next-line @typescript-eslint/naming-convention
  'data-testid'?: string;
}

export function SettingsSelect<T extends string>(props: SettingsSelectProps<T>) {
  const [value, setValue] = useState<T>(props.defaultValue ?? props.items[0]?.value ?? '');
  const [dropdownVisible, , closeDropdown, toggleDropdown] = useBoolean();

  // When typing to search the current search value is stored here.
  const searchRef = useRef<string>('');
  // Scheduler for clearing the search string after the user has stopped typing.
  const searchClearScheduler = useScheduler();

  const onSelect = useCallback((value: T) => {
    setValue(value);
    closeDropdown();
  }, []);

  // Handle keyboard shortcuts and type search
  const onKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLDivElement>) => {
      switch (event.key) {
        case 'ArrowUp':
          setValue((prevValue) => findPreviousValue(props.items, prevValue));
          break;
        case 'ArrowDown':
          setValue((prevValue) => findNextValue(props.items, prevValue));
          break;
        case 'Home':
          setValue(props.items[0]?.value ?? '');
          break;
        case 'End':
          setValue(props.items[props.items.length - 1]?.value ?? '');
          break;
        default:
          // Only accept printable characters for text search.
          if (event.key.length === 1) {
            searchClearScheduler.cancel();
            searchRef.current += event.key.toLowerCase();
            searchClearScheduler.schedule(() => (searchRef.current = ''), 500);

            setValue((prevValue) => findSearchedValue(props.items, prevValue, searchRef.current));
          }
          break;
      }
    },
    [props.items],
  );

  // Update the parent when the value changes.
  useEffect(() => {
    props.onUpdate(value);
  }, [value]);

  return (
    <AriaInput>
      <StyledSelect onBlur={closeDropdown} onKeyDown={onKeyDown} role="listbox">
        <StyledSelectedContainer data-testid={props['data-testid']} onClick={toggleDropdown}>
          <StyledSelectedContainerInner>
            <StyledSelectedText>
              {props.items.find((item) => item.value === value)?.label ?? ''}
            </StyledSelectedText>
            <StyledChevron tintColor={colors.white60} source="icon-chevron-down" width={22} />
          </StyledSelectedContainerInner>
          <StyledInvisibleItems>
            {props.items.map((item) => (
              <StyledInvisibleItemsInner key={item.label}>{item.label}</StyledInvisibleItemsInner>
            ))}
          </StyledInvisibleItems>
        </StyledSelectedContainer>
        {dropdownVisible && (
          <StyledItems $direction={props.direction ?? 'down'}>
            <CustomScrollbars>
              {props.items.map((item) => (
                <Item
                  key={item.value}
                  item={item}
                  selected={item.value === value}
                  onSelect={onSelect}
                />
              ))}
            </CustomScrollbars>
          </StyledItems>
        )}
      </StyledSelect>
    </AriaInput>
  );
}

function findPreviousValue<T extends string>(
  items: Array<SettingsSelectItem<T>>,
  currentValue: T,
): T {
  const currentIndex = items.findIndex((item) => item.value === currentValue) ?? 0;
  const newIndex = Math.max(currentIndex - 1, 0);
  return items[newIndex]?.value ?? '';
}

function findNextValue<T extends string>(items: Array<SettingsSelectItem<T>>, currentValue: T): T {
  const currentIndex = items.findIndex((item) => item.value === currentValue) ?? 0;
  const newIndex = Math.min(currentIndex + 1, items.length - 1);
  return items[newIndex]?.value ?? '';
}

function findSearchedValue<T extends string>(
  items: Array<SettingsSelectItem<T>>,
  currentValue: T,
  searchValue: string,
): T {
  const currentIndex = items.findIndex((item) => item.value === currentValue) ?? 0;
  const itemsFromCurrent = [...items.slice(currentIndex + 1), ...items.slice(0, currentIndex)];
  const searchedValue = itemsFromCurrent.find((item) =>
    item.label.toLowerCase().startsWith(searchValue),
  );

  return searchedValue?.value ?? currentValue;
}

const StyledItem = styled.div<{ $selected: boolean }>((props) => ({
  display: 'flex',
  alignItems: 'center',
  borderRadius: '4px',
  lineHeight: '22px',
  paddingLeft: props.$selected ? '0px' : '23px',
  paddingRight: '18px',
  whiteSpace: 'nowrap',
  '&&:hover': {
    backgroundColor: colors.blue,
  },
}));

const TickIcon = styled(ImageView)({
  marginLeft: '5px',
  marginRight: '6px',
});

interface ItemProps<T extends string> {
  item: SettingsSelectItem<T>;
  selected: boolean;
  onSelect: (key: T) => void;
}

function Item<T extends string>(props: ItemProps<T>) {
  const onClick = useCallback(() => {
    props.onSelect(props.item.value);
  }, [props.onSelect, props.item.value]);

  return (
    <StyledItem
      onClick={onClick}
      role="option"
      $selected={props.selected}
      aria-selected={props.selected}>
      {props.selected && <TickIcon tintColor={colors.white} source="icon-tick" width={12} />}
      {props.item.label}
    </StyledItem>
  );
}
