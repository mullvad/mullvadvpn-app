import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import { useSelectLocationViewContext } from '../../../../SelectLocationViewContext';
import {
  useEffectSetIsolatedItem,
  useEffectSetSearching,
  useEffectSetSearchTerm,
  useHandleInputKeyDown,
} from './hooks';
import {
  SelectLocationSelectorItemProvider,
  useSelectLocationSelectorItemContext,
} from './SelectLocationSelectorItemContext';

export type SelectLocationSelectorItemProps = LocationSelectorItemProps & {
  defaultValue?: string;
  placeholder?: string;
};

const StyledInput = styled(LocationSelector.Items.Item.TextField.Input)`
  &&::-webkit-search-cancel-button {
    display: none;
  }
`;

function SelectLocationSelectorItemImpl({
  id,
  placeholder,
  ...props
}: Omit<SelectLocationSelectorItemProps, 'value' | 'inputRef' | 'delay'>) {
  const {
    triggerRef,
    focused,
    setFocused,
    textField: {
      inputRef,
      value,
      handleOnValueChange: textFieldHandleValueChange,
      handleFocus,
      reset,
    },
  } = useSelectLocationSelectorItemContext();
  const { searchTerm } = useSelectLocationViewContext();

  useEffectSetSearching();
  useEffectSetSearchTerm();
  useEffectSetIsolatedItem(id);

  const handleTextFieldValueChange = React.useCallback(
    (_: string, value: string) => {
      textFieldHandleValueChange(value);
    },
    [textFieldHandleValueChange],
  );

  const handleClearButtonClick = React.useCallback(() => {
    reset();
    setFocused(false);
    triggerRef.current?.focus();
  }, [reset, setFocused, triggerRef]);

  const handleKeyDown = useHandleInputKeyDown();

  const handleFocusExit = React.useCallback(() => {
    if (searchTerm?.length < 2) {
      reset();
    }

    setFocused(false);
  }, [reset, searchTerm?.length, setFocused]);

  return (
    <LocationSelector.Items.Item id={id} inputRef={inputRef} triggerRef={triggerRef} {...props}>
      <LocationSelector.Items.Item.TextField
        value={value}
        onFocusExit={handleFocusExit}
        onValueChange={handleTextFieldValueChange}>
        <StyledInput
          placeholder={placeholder}
          onFocus={handleFocus}
          onKeyDown={handleKeyDown}
          type="search"
        />
        <LocationSelector.Items.Item.TextField.ClearButton
          visible={focused}
          onClick={handleClearButtonClick}
          aria-label={messages.gettext('Clear')}
        />
      </LocationSelector.Items.Item.TextField>
    </LocationSelector.Items.Item>
  );
}

export function SelectLocationSelectorItem({
  defaultValue,
  ...props
}: SelectLocationSelectorItemProps) {
  return (
    <SelectLocationSelectorItemProvider defaultValue={defaultValue}>
      <SelectLocationSelectorItemImpl {...props} />
    </SelectLocationSelectorItemProvider>
  );
}
