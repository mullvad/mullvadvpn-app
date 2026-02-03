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
  const validateValue = React.useCallback(
    (value: string) => {
      const result = validate ? validate(value) : true;
      if (typeof result === 'string') {
        return { invalid: true, invalidReason: result };
      } else {
        return { invalid: !result, invalidReason: null };
      }
    },
    [validate],
  );

  const { invalid: initialInvalid, invalidReason: initialInvalidReason } = validateValue(value);
  const [invalid, setInvalid] = React.useState(initialInvalid);
  const [invalidReason, setInvalidReason] = React.useState<string | null>(initialInvalidReason);
  const [dirty, setDirty] = React.useState(false);

  const reset = React.useCallback(
    (resetValue?: string) => {
      let newValue = '';
      if (resetValue !== undefined) {
        newValue = resetValue;
      } else if (defaultValue !== undefined) {
        newValue = defaultValue;
      }

      const { invalid, invalidReason } = validateValue(newValue);

      setValue(newValue);
      setInvalid(invalid);
      setInvalidReason(invalidReason);
      setDirty(false);
    },
    [defaultValue, validateValue],
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
      const { invalid, invalidReason } = validateValue(formattedValue);

      setInvalid(invalid);
      setInvalidReason(invalidReason);
      setValue(formattedValue);
      setDirty(true);
    },
    [format, validateValue],
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
