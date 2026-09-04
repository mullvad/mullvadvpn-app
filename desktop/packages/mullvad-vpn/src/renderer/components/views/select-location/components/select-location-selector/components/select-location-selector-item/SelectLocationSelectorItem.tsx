import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import {
  useHandleClearButtonClick,
  useHandleFocusExit,
  useHandleInputKeyDown,
  useHandleValueChange,
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
    textField: { inputRef, value, handleFocus },
  } = useSelectLocationSelectorItemContext();

  const handleClearButtonClick = useHandleClearButtonClick();

  const handleKeyDown = useHandleInputKeyDown();
  const handleFocusExit = useHandleFocusExit();
  const handleValueChange = useHandleValueChange();

  const showClearButton = focused && value.length > 0;

  return (
    <LocationSelector.Items.Item id={id} inputRef={inputRef} triggerRef={triggerRef} {...props}>
      <LocationSelector.Items.Item.TextField
        value={value}
        onFocusExit={handleFocusExit}
        onValueChange={handleValueChange}>
        <StyledInput
          placeholder={placeholder}
          onFocus={handleFocus}
          onKeyDown={handleKeyDown}
          type="search"
        />
        {showClearButton && (
          <LocationSelector.Items.Item.TextField.ClearButton
            onClick={handleClearButtonClick}
            aria-label={messages.gettext('Clear')}
          />
        )}
      </LocationSelector.Items.Item.TextField>
    </LocationSelector.Items.Item>
  );
}

export function SelectLocationSelectorItem({
  defaultValue,
  id,
  ...props
}: SelectLocationSelectorItemProps) {
  return (
    <SelectLocationSelectorItemProvider id={id} defaultValue={defaultValue}>
      <SelectLocationSelectorItemImpl id={id} {...props} />
    </SelectLocationSelectorItemProvider>
  );
}
