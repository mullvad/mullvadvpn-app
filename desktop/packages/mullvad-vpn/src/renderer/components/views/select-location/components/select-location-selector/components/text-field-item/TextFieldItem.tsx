import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorTextFieldItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import {
  useHandleClearButtonClick,
  useHandleFocusExit,
  useHandleInputKeyDown,
  useHandleValueChange,
} from './hooks';
import { TextFieldItemProvider, useTextFieldItemContext } from './TextFieldItemContext';

export type SelectLocationSelectorItemProps = LocationSelectorTextFieldItemProps & {
  defaultValue?: string;
  placeholder?: string;
};

const StyledInput = styled(LocationSelector.Items.TextFieldItem.TextField.Input)`
  &&::-webkit-search-cancel-button {
    display: none;
  }
`;

function TextFieldItemImpl({
  id,
  placeholder,
  ...props
}: Omit<SelectLocationSelectorItemProps, 'value' | 'inputRef' | 'delay'>) {
  const {
    triggerRef,
    focused,
    textField: { inputRef, value, handleFocus },
  } = useTextFieldItemContext();

  const handleClearButtonClick = useHandleClearButtonClick();

  const handleKeyDown = useHandleInputKeyDown();
  const handleFocusExit = useHandleFocusExit();
  const handleValueChange = useHandleValueChange();

  const showClearButton = focused && value.length > 0;

  return (
    <LocationSelector.Items.TextFieldItem
      id={id}
      inputRef={inputRef}
      triggerRef={triggerRef}
      {...props}>
      <LocationSelector.Items.TextFieldItem.TextField
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
          <LocationSelector.Items.TextFieldItem.TextField.ClearButton
            onClick={handleClearButtonClick}
            aria-label={messages.gettext('Clear')}
          />
        )}
      </LocationSelector.Items.TextFieldItem.TextField>
    </LocationSelector.Items.TextFieldItem>
  );
}

export function TextFieldItem({ defaultValue, id, ...props }: SelectLocationSelectorItemProps) {
  return (
    <TextFieldItemProvider id={id} defaultValue={defaultValue}>
      <TextFieldItemImpl id={id} {...props} />
    </TextFieldItemProvider>
  );
}
