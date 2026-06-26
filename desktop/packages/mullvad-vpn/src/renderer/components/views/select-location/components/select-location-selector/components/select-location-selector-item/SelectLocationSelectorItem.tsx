import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import {
  useEffectSetIsolatedItem,
  useEffectSetSearching,
  useEffectSetSearchTerm,
  useHandleInputBlur,
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
    searching,
    textField: {
      inputRef,
      value,
      handleOnValueChange: textFieldHandleValueChange,
      handleFocus,
      reset,
    },
  } = useSelectLocationSelectorItemContext();

  const handleInputBlur = useHandleInputBlur();

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
  }, [reset]);

  const handleKeyDown = useHandleInputKeyDown();

  return (
    <LocationSelector.Items.Item id={id} inputRef={inputRef} triggerRef={triggerRef} {...props}>
      <LocationSelector.Items.Item.TextField
        value={value}
        onValueChange={handleTextFieldValueChange}>
        <StyledInput
          placeholder={placeholder}
          onFocus={handleFocus}
          onBlur={handleInputBlur}
          onKeyDown={handleKeyDown}
          type="search"
        />
        <LocationSelector.Items.Item.TextField.ClearButton
          visible={searching}
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
