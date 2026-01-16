import React from 'react';

export type UseTextFieldProps = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  defaultValue?: string;
  validate?: (value: string) => boolean;
  format?: (value: string) => string;
};

export type UseTextFieldState = {
  value: string;
  invalid: boolean;
  dirty: boolean;
  reset: () => void;
  focus: () => void;
  blur: () => void;
  handleOnValueChange: (newValue: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
};

export function useTextField({
  inputRef,
  defaultValue,
  format,
  validate,
}: UseTextFieldProps): UseTextFieldState {
  const [value, setValue] = React.useState(defaultValue ?? '');
  const [invalid, setInvalid] = React.useState(validate ? !validate(value) : false);
  const [dirty, setDirty] = React.useState(false);

  const reset = React.useCallback(() => {
    const newValue = defaultValue ?? '';
    setValue(newValue);
    setInvalid(validate ? !validate(newValue) : false);
    setDirty(false);
  }, [defaultValue, validate]);

  const focus = React.useCallback(() => {
    inputRef.current?.focus();
  }, [inputRef]);

  const blur = React.useCallback(() => {
    inputRef.current?.blur();
  }, [inputRef]);

  const handleOnValueChange = React.useCallback(
    (newValue: string) => {
      const formattedValue = format ? format(newValue) : newValue;
      const invalid = validate ? !validate(formattedValue) : false;
      setInvalid(invalid);
      setValue(formattedValue);
      setDirty(true);
    },
    [format, validate],
  );

  return { value, invalid, dirty, reset, blur, focus, handleOnValueChange, inputRef };
}
