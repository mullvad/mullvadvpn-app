import React from 'react';

export type UseTextFieldProps = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  defaultValue?: string;
  validate?: (value: string) => boolean | string;
  format?: (value: string) => string;
};

export type UseTextFieldState = {
  value: string;
  invalid: boolean;
  invalidReason: string | null;
  dirty: boolean;
  reset: (value?: string) => void;
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
  const [invalidReason, setInvalidReason] = React.useState<string | null>(null);
  const [dirty, setDirty] = React.useState(false);

  const reset = React.useCallback(
    (resetValue?: string) => {
      let newValue = '';
      if (resetValue !== undefined) {
        newValue = resetValue;
      } else if (defaultValue !== undefined) {
        newValue = defaultValue;
      }
      setValue(newValue);
      setInvalid(validate ? !validate(newValue) : false);
      setInvalidReason(null);
      setDirty(false);
    },
    [defaultValue, validate],
  );

  const focus = React.useCallback(() => {
    inputRef.current?.focus();
  }, [inputRef]);

  const blur = React.useCallback(() => {
    inputRef.current?.blur();
  }, [inputRef]);

  const handleOnValueChange = React.useCallback(
    (newValue: string) => {
      const formattedValue = format ? format(newValue) : newValue;
      const validationResult = validate ? validate(formattedValue) : true;
      const invalid = typeof validationResult === 'string' ? true : !validationResult;
      const invalidReason = typeof validationResult === 'string' ? validationResult : null;

      setInvalid(invalid);
      setInvalidReason(invalidReason);
      setValue(formattedValue);
      setDirty(true);
    },
    [format, validate],
  );

  return {
    value,
    invalid,
    invalidReason,
    dirty,
    reset,
    blur,
    focus,
    handleOnValueChange,
    inputRef,
  };
}
