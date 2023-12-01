import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { AriaInput } from '../AriaGroup';
import { smallNormalText } from '../common-styles';
import { useSettingsFormSubmittableReporter } from './SettingsForm';
import { useSettingsRowContext } from './SettingsRow';

const StyledInput = styled.input(smallNormalText, {
  flex: 1,
  textAlign: 'right',
  background: 'transparent',
  border: 'none',
  color: colors.white,
  width: '100px',

  '&&::placeholder': {
    color: colors.white50,
  },
});

interface SettingsTextInputProps extends InputProps<'text'> {
  defaultValue?: string;
}

export function SettingsTextInput(props: SettingsTextInputProps) {
  return <Input type="text" {...props} />;
}

interface SettingsNumberInputProps
  extends Omit<InputProps<'number'>, 'onUpdate' | 'validate' | 'value'> {
  defaultValue?: number;
  value?: number | '';
  onUpdate: (value: number | undefined) => void;
  validate?: (value: number) => boolean;
}

// NumberInput is basically a text input but it parses all values as numbers.
export function SettingsNumberInput(props: SettingsNumberInputProps) {
  const { onUpdate, validate, value, ...otherProps } = props;

  const parse = useCallback((value: string) => {
    const parsedValue = parseInt(value);
    return isNaN(parsedValue) ? undefined : parsedValue;
  }, []);

  const onNumberUpdate = useCallback(
    (value: string) => {
      onUpdate(parse(value));
    },
    [onUpdate],
  );

  const validateNumber = useCallback(
    (value: string) => {
      const parsedValue = parse(value);
      return (parsedValue === undefined || validate?.(parsedValue)) ?? true;
    },
    [validate],
  );

  return (
    <Input
      {...otherProps}
      value={value ?? ''}
      onUpdate={onNumberUpdate}
      validate={validateNumber}
    />
  );
}

type ValueTypes = 'text' | 'number';
type ValueType<T extends ValueTypes> = T extends 'number' ? number | '' : string;

interface InputProps<T extends ValueTypes> extends React.HTMLAttributes<HTMLInputElement> {
  type?: T;
  value?: ValueType<T>;
  defaultValue?: ValueType<T>;
  onUpdate: (value: string) => void;
  validate?: (value: string) => boolean;
  optionalInForm?: boolean;
}

function Input<T extends ValueTypes>(props: InputProps<T>) {
  const { onUpdate, onChange: propsOnChange, validate, optionalInForm, ...otherProps } = props;
  const reportSubmittable = useSettingsFormSubmittableReporter();

  const { setInvalid } = useSettingsRowContext();

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value;

      // Report change to parent
      propsOnChange?.(event);
      onUpdate(value);

      if (validate?.(value) === false && value !== '') {
        // Report validity and submittability to settings row context and form context.
        setInvalid(true);
        reportSubmittable(false);
      } else {
        setInvalid(false);
        reportSubmittable(value !== '' || optionalInForm === true);
      }
    },
    [onUpdate, propsOnChange, validate, optionalInForm],
  );

  // Report submittability to form context on load.
  useEffect(() => {
    const value = props.value ?? props.defaultValue ?? '';
    reportSubmittable(
      (value !== '' || optionalInForm === true) && validate?.(`${value}`) !== false,
    );
  }, []);

  return (
    <AriaInput>
      <StyledInput {...otherProps} onChange={onChange} />
    </AriaInput>
  );
}
