import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorTextFieldItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import { FilterTrailingButton } from './components';
import {
  useHandleClearButtonClick,
  useHandleFocusExit,
  useHandleInputKeyDown,
  useHandleValueChange,
} from './hooks';
import { TextFieldItemProvider, useTextFieldItemContext } from './TextFieldItemContext';

export type SelectLocationSelectorItemProps = LocationSelectorTextFieldItemProps & {
  defaultValue?: string;
  invalid?: boolean;
  placeholder?: string;
};

const StyledInput = styled(LocationSelector.Items.TextFieldItem.TextField.Input)`
  &&::-webkit-search-cancel-button {
    display: none;
  }
`;

function TextFieldItemImpl({
  id,
  invalid,
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

  const showClearButton = true; // focused && value.length > 0;
  const showSupportingText = invalid;

  return (
    <LocationSelector.Items.TextFieldItem
      id={id}
      inputRef={inputRef}
      triggerRef={triggerRef}
      {...props}>
      <LocationSelector.Items.TextFieldItem.TextField
        invalid={invalid}
        value={value}
        onFocusExit={handleFocusExit}
        onValueChange={handleValueChange}>
        <LocationSelector.Items.TextFieldItem.TextField.TextArea>
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
          <FilterTrailingButton />
        </LocationSelector.Items.TextFieldItem.TextField.TextArea>
        {!showSupportingText && (
          <LocationSelector.Items.TextFieldItem.TextField.SupportingText>
            test
          </LocationSelector.Items.TextFieldItem.TextField.SupportingText>
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
