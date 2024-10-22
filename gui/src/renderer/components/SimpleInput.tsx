import { useCallback, useState } from 'react';
import React from 'react';
import styled from 'styled-components';

import { useCombinedRefs } from '../lib/utility-hooks';
import { normalText } from './common-styles';

const StyledInput = styled.input.attrs({ type: 'text' })(normalText, {
  padding: '6px 8px',
  borderRadius: '4px',
  outline: 0,
  border: 0,
  lineHeight: '21px',
});

interface SimpleInputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  onChangeValue?: (value: string) => void;
  onSubmitValue?: (value: string) => void;
}

function SimpleInput(props: SimpleInputProps, ref: React.Ref<HTMLInputElement>) {
  const {
    onChangeValue,
    onSubmitValue,
    onChange: propsOnChange,
    onSubmit: propsOnSubmit,
    onKeyPress: propsOnKeyPress,
    ...otherProps
  } = props;
  const [value, setValue] = useState((props.value as string) ?? '');

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      setValue(event.target.value);
      propsOnChange?.(event);
      onChangeValue?.(event.target.value);
    },
    [propsOnChange, onChangeValue],
  );

  const onSubmit = useCallback(
    (event: React.FormEvent<HTMLInputElement>) => {
      propsOnSubmit?.(event);
      onSubmitValue?.(value);
    },
    [propsOnSubmit, onSubmitValue, value],
  );

  const onKeyPress = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      propsOnKeyPress?.(event);
      if (event.key === 'Enter') {
        onSubmitValue?.(value);
      }
    },
    [propsOnKeyPress, onSubmitValue, value],
  );

  const refCallback = useCallback(
    (element: HTMLInputElement | null) => {
      if (element && otherProps.autoFocus) {
        setTimeout(() => element.focus());
      }
    },
    [otherProps.autoFocus],
  );

  const combinedRef = useCombinedRefs(refCallback, ref);

  return (
    <StyledInput
      {...otherProps}
      ref={combinedRef}
      onChange={onChange}
      onSubmit={onSubmit}
      onKeyPress={onKeyPress}
    />
  );
}

export default React.forwardRef(SimpleInput);
