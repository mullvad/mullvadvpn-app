import React from 'react';

import { useDebounce } from '../../../../hooks/use-debounce';

export type UseTextFieldProps = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  defaultValue?: string;
  validate?: (value: string) => boolean | string;
  format?: (value: string) => string;
  delay?: number;
};

export type UseTextFieldState = {
  value: string;
  debouncedValue: string;
  invalid: boolean;
  invalidReason: string | null;
  dirty: boolean;
  touched: boolean;
  reset: (value?: string) => void;
  focus: () => void;
  blur: () => void;
  handleOnValueChange: (newValue: string) => void;
  handleFocus: () => void;
  handleBlur: () => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
};

export function useTextField({
  inputRef,
  defaultValue,
  format,
  validate,
  delay = 0,
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
  const [touched, setTouched] = React.useState(false);
  const [dirty, setDirty] = React.useState(false);
  const debouncedValue = useDebounce(value, delay);

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
      setTouched(false);
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
      if (formattedValue !== defaultValue) {
        setDirty(true);
      } else {
        setDirty(false);
      }
    },
    [defaultValue, format, validateValue],
  );

  const handleFocus = React.useCallback(() => {
    setTouched(true);
  }, []);

  const handleBlur = React.useCallback(() => {
    setTouched(false);
  }, []);

  return {
    value,
    debouncedValue,
    invalid,
    invalidReason,
    dirty,
    touched,
    reset,
    blur,
    focus,
    handleOnValueChange,
    handleFocus,
    handleBlur,
    inputRef,
  };
}
