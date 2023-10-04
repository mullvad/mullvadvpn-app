import { useCallback, useState } from 'react';
import styled from 'styled-components';

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

export default function SimpleInput(props: SimpleInputProps) {
  const { onChangeValue, onSubmitValue, ...otherProps } = props;
  const [value, setValue] = useState((props.value as string) ?? '');

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      setValue(event.target.value);
      otherProps.onChange?.(event);
      onChangeValue?.(event.target.value);
    },
    [otherProps.onChange, onChangeValue],
  );

  const onSubmit = useCallback(
    (event: React.FormEvent<HTMLInputElement>) => {
      otherProps.onSubmit?.(event);
      onSubmitValue?.(value);
    },
    [otherProps.onSubmit, onSubmitValue, value],
  );

  const onKeyPress = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      props.onKeyPress?.(event);
      if (event.key === 'Enter') {
        onSubmitValue?.(value);
      }
    },
    [props.onKeyPress, onSubmitValue, value],
  );

  const refCallback = useCallback((element: HTMLInputElement | null) => {
    if (element && otherProps.autoFocus) {
      setTimeout(() => element.focus());
    }
  }, []);

  return (
    <StyledInput
      {...otherProps}
      ref={refCallback}
      onChange={onChange}
      onSubmit={onSubmit}
      onKeyPress={onKeyPress}
    />
  );
}
