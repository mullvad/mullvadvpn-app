import React, { useCallback, useContext, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { useBoolean, useCombinedRefs } from '../../lib/utilityHooks';
import { normalText } from '../common-styles';
import ImageView from '../ImageView';
import { BackAction } from '../KeyboardNavigation';
import StandaloneSwitch from '../Switch';
import { CellDisabledContext, Container } from './Container';

export const Switch = React.forwardRef(function SwitchT(
  props: StandaloneSwitch['props'],
  ref: React.Ref<StandaloneSwitch>,
) {
  const disabled = useContext(CellDisabledContext);
  return <StandaloneSwitch ref={ref} disabled={disabled} {...props} />;
});

const inputTextStyles: React.CSSProperties = {
  ...normalText,
  height: '18px',
  textAlign: 'right',
  padding: '0px',
};

const StyledInput = styled.input({}, (props: { focused: boolean; valid?: boolean }) => ({
  ...inputTextStyles,
  backgroundColor: 'transparent',
  border: 'none',
  width: '100%',
  height: '100%',
  color: props.valid === false ? colors.red : props.focused ? colors.blue : colors.white,
  '::placeholder': {
    color: props.focused ? colors.blue60 : colors.white60,
  },
}));

interface IInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  value?: string;
  initialValue?: string;
  validateValue?: (value: string) => boolean;
  modifyValue?: (value: string) => string;
  submitOnBlur?: boolean;
  onSubmitValue?: (value: string) => void;
  onInvalidValue?: (value: string) => void;
  onChangeValue?: (value: string) => void;
}

// If value is provided this component behaves like a controlled component.
// If value isn't provided, then initialValue will be used for the initial value, but updates to
// initialValue will also cause the internal value to update.
function InputWithRef(props: IInputProps, forwardedRef: React.Ref<HTMLInputElement>) {
  const {
    initialValue,
    validateValue,
    modifyValue,
    submitOnBlur,
    onSubmitValue,
    onInvalidValue,
    onChangeValue,
    ...otherProps
  } = props;

  const [isFocused, setFocused, setBlurred] = useBoolean(false);

  // internalValue will be used when the component is uncontrolled.
  const [internalValue, setInternalValue] = useState(props.value ?? props.initialValue ?? '');
  const value = props.value ?? internalValue;

  const inputRef = useRef() as React.RefObject<HTMLInputElement>;
  const combinedRef = useCombinedRefs(inputRef, forwardedRef);

  const onSubmit = useCallback(
    (value: string) => {
      if (validateValue?.(value) !== false) {
        onSubmitValue?.(value);
      } else {
        onInvalidValue?.(value);
      }
    },
    [onSubmitValue, onInvalidValue],
  );

  const onFocus = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setFocused();
      props.onFocus?.(event);
    },
    [props.onFocus],
  );

  const onBlur = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setBlurred();
      props.onBlur?.(event);
      if (submitOnBlur) {
        onSubmit(value);
      }
    },
    [value, props.onBlur, validateValue, onSubmit, submitOnBlur],
  );

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = modifyValue?.(event.target.value) ?? event.target.value;
      if (props.value === undefined) {
        // Only update the internal value when in uncontrolled mode to not cause unnecessary render
        // cycles.
        setInternalValue(value);
      }

      props.onChange?.(event);
      onChangeValue?.(value);
    },
    [modifyValue, props.onSubmit],
  );

  const onKeyPress = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        onSubmit(value);
        inputRef.current?.blur();
      }
      props.onKeyPress?.(event);
    },
    [value, onSubmit, inputRef, props.onKeyPress],
  );

  // If the the initialValue changes in the uncontrolled mode when the user isn't currently writing,
  // then we want to update the value.
  useEffect(() => {
    if (
      !isFocused &&
      props.value === undefined &&
      props.initialValue !== undefined &&
      internalValue !== props.initialValue
    ) {
      setInternalValue(props.initialValue);
      onChangeValue?.(props.initialValue);
    }
  }, [props.initialValue]);

  const valid = validateValue?.(value);

  return (
    <CellDisabledContext.Consumer>
      {(disabled) => (
        <StyledInput
          {...otherProps}
          ref={combinedRef}
          type="text"
          valid={valid}
          focused={isFocused}
          aria-invalid={!valid}
          onChange={onChange}
          onFocus={onFocus}
          onBlur={onBlur}
          onKeyPress={onKeyPress}
          value={value}
          disabled={disabled}
        />
      )}
    </CellDisabledContext.Consumer>
  );
}

export const Input = React.memo(React.forwardRef(InputWithRef));

const InputFrame = styled.div((props: { focused: boolean }) => ({
  display: 'flex',
  flexGrow: 0,
  backgroundColor: props.focused ? colors.white : 'rgba(255,255,255,0.1)',
  borderRadius: '4px',
  padding: '6px 8px',
}));

const StyledAutoSizingTextInputContainer = styled.div({
  position: 'relative',
});

const StyledAutoSizingTextInputFiller = styled.pre({
  ...inputTextStyles,
  minWidth: '80px',
  color: 'transparent',
});

const StyledAutoSizingTextInputWrapper = styled.div({
  position: 'absolute',
  top: '0px',
  left: '0px',
  width: '100%',
  height: '100%',
});

function AutoSizingTextInputWithRef(props: IInputProps, forwardedRef: React.Ref<HTMLInputElement>) {
  const { onFocus, onBlur, ...otherProps } = props;

  const [focused, setFocused, setBlurred] = useBoolean(false);
  const inputRef = useRef() as React.RefObject<HTMLInputElement>;
  const combinedRef = useCombinedRefs(inputRef, forwardedRef);

  const onBlurWrapper = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setBlurred();
      onBlur?.(event);
    },
    [onBlur],
  );

  const onFocusWrapper = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setFocused();
      onFocus?.(event);
    },
    [onFocus],
  );

  const blur = useCallback(() => inputRef.current?.blur(), []);

  const value = inputRef.current?.value;

  return (
    <BackAction disabled={!focused} action={blur}>
      <InputFrame focused={focused}>
        <StyledAutoSizingTextInputContainer>
          <StyledAutoSizingTextInputWrapper>
            <Input
              ref={combinedRef}
              onBlur={onBlurWrapper}
              onFocus={onFocusWrapper}
              {...otherProps}
            />
          </StyledAutoSizingTextInputWrapper>
          <StyledAutoSizingTextInputFiller className={otherProps.className} aria-hidden={true}>
            {value === '' ? otherProps.placeholder : value}
          </StyledAutoSizingTextInputFiller>
        </StyledAutoSizingTextInputContainer>
      </InputFrame>
    </BackAction>
  );
}

export const AutoSizingTextInput = React.memo(React.forwardRef(AutoSizingTextInputWithRef));

const StyledCellInputRowContainer = styled(Container)({
  backgroundColor: 'white',
  marginBottom: '1px',
});

const StyledSubmitButton = styled.button({
  border: 'none',
  backgroundColor: 'transparent',
  padding: '10px 0',
});

const StyledInputWrapper = styled.div(normalText, (props: { marginLeft: number }) => ({
  position: 'relative',
  flex: 1,
  width: '171px',
  marginLeft: props.marginLeft + 'px',
  lineHeight: '24px',
  minHeight: '24px',
  fontWeight: 400,
  padding: '10px 0',
  maxWidth: '100%',
}));

const StyledTextArea = styled.textarea(normalText, (props: { invalid?: boolean }) => ({
  position: 'absolute',
  top: 0,
  left: 0,
  width: '100%',
  height: '100%',
  backgroundColor: 'transparent',
  border: 'none',
  flex: 1,
  lineHeight: '24px',
  fontWeight: 400,
  resize: 'none',
  padding: '10px 25px 10px 0',
  color: props.invalid ? colors.red : 'auto',
}));

const StyledInputFiller = styled.div({
  whiteSpace: 'pre-wrap',
  overflowWrap: 'break-word',
  minHeight: '24px',
  color: 'transparent',
  marginRight: '25px',
});

interface IRowInputProps {
  initialValue?: string;
  onChange?: (value: string) => void;
  onSubmit: (value: string) => void;
  onFocus?: (event: React.FocusEvent<HTMLTextAreaElement>) => void;
  onBlur?: (event?: React.FocusEvent<HTMLTextAreaElement>) => void;
  paddingLeft?: number;
  invalid?: boolean;
  autofocus?: boolean;
  placeholder?: string;
}

export function RowInput(props: IRowInputProps) {
  const [value, setValue] = useState(props.initialValue ?? '');
  const textAreaRef = useRef() as React.RefObject<HTMLTextAreaElement>;
  const [focused, setFocused, setBlurred] = useBoolean(false);

  const submit = useCallback(() => props.onSubmit(value), [props.onSubmit, value]);
  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLTextAreaElement>) => {
      const value = event.target.value;
      setValue(value);
      props.onChange?.(value);
    },
    [props.onChange],
  );
  const onKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        submit();
      }
    },
    [submit],
  );

  const onFocus = useCallback(
    (event: React.FocusEvent<HTMLTextAreaElement>) => {
      setFocused();
      props.onFocus?.(event);
    },
    [props.onFocus],
  );
  const onBlur = useCallback(
    (event: React.FocusEvent<HTMLTextAreaElement>) => {
      setBlurred();
      props.onBlur?.(event);
    },
    [props.onBlur],
  );

  const focus = useCallback(() => {
    const input = textAreaRef.current;
    if (input) {
      input.focus();
      input.selectionStart = input.selectionEnd = value.length;
    }
  }, [textAreaRef, value.length]);

  const blur = useCallback(() => textAreaRef.current?.blur(), []);

  useEffect(() => {
    if (props.autofocus) {
      focus();
    }
  }, []);

  useEffect(() => {
    if (props.invalid) {
      focus();
    }
  }, [props.invalid, focus]);

  return (
    <BackAction disabled={!focused} action={blur}>
      <StyledCellInputRowContainer>
        <StyledInputWrapper marginLeft={props.paddingLeft ?? 0}>
          <StyledInputFiller>{value}</StyledInputFiller>
          <StyledTextArea
            ref={textAreaRef}
            onChange={onChange}
            onKeyDown={onKeyDown}
            rows={1}
            value={value}
            invalid={props.invalid}
            onFocus={onFocus}
            onBlur={onBlur}
            placeholder={props.placeholder}
          />
        </StyledInputWrapper>
        <StyledSubmitButton onClick={submit}>
          <ImageView
            source="icon-check"
            height={18}
            tintColor={value === '' ? colors.blue60 : colors.blue}
            tintHoverColor={value === '' ? colors.blue60 : colors.blue80}
          />
        </StyledSubmitButton>
      </StyledCellInputRowContainer>
    </BackAction>
  );
}
