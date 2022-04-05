import React, { useCallback, useContext, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { useBoolean } from '../../lib/utilityHooks';
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
  validateValue?: (value: string) => boolean;
  modifyValue?: (value: string) => string;
  submitOnBlur?: boolean;
  onSubmitValue?: (value: string) => void;
  onChangeValue?: (value: string) => void;
}

interface IInputState {
  value?: string;
  focused: boolean;
}

export class Input extends React.Component<IInputProps, IInputState> {
  public state = {
    value: this.props.value ?? '',
    focused: false,
  };

  public inputRef = React.createRef<HTMLInputElement>();

  public componentDidUpdate(prevProps: IInputProps, _prevState: IInputState) {
    if (
      !this.state.focused &&
      prevProps.value !== this.props.value &&
      this.props.value !== this.state.value
    ) {
      this.setState(
        (_state, props) => ({
          value: props.value,
        }),
        () => {
          this.props.onChangeValue?.(this.state.value);
        },
      );
    }
  }

  public render() {
    const {
      type: _type,
      onChange: _onChange,
      onFocus: _onFocus,
      onBlur: _onBlur,
      onKeyPress: _onKeyPress,
      value: _value,
      modifyValue: _modifyValue,
      submitOnBlur: _submitOnBlur,
      onChangeValue: _onChangeValue,
      onSubmitValue: _onSubmitValue,
      validateValue,
      ...otherProps
    } = this.props;

    const valid = validateValue?.(this.state.value);

    return (
      <CellDisabledContext.Consumer>
        {(disabled) => (
          <StyledInput
            ref={this.inputRef}
            type="text"
            valid={valid}
            focused={this.state.focused}
            aria-invalid={!valid}
            onChange={this.onChange}
            onFocus={this.onFocus}
            onBlur={this.onBlur}
            onKeyPress={this.onKeyPress}
            value={this.state.value}
            disabled={disabled}
            {...otherProps}
          />
        )}
      </CellDisabledContext.Consumer>
    );
  }

  public blur = () => {
    this.inputRef.current?.blur();
  };

  private onChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const value = this.props.modifyValue?.(event.target.value) ?? event.target.value;
    this.setState({ value });
    this.props.onChange?.(event);
    this.props.onChangeValue?.(value);
  };

  private onFocus = (event: React.FocusEvent<HTMLInputElement>) => {
    this.setState({ focused: true });
    this.props.onFocus?.(event);
  };

  private onBlur = (event: React.FocusEvent<HTMLInputElement>) => {
    this.setState({ focused: false });

    this.props.onBlur?.(event);
    if (this.props.validateValue?.(this.state.value) !== false && this.props.submitOnBlur) {
      this.props.onSubmitValue?.(this.state.value);
    } else {
      this.setState({ value: this.props.value });
    }
  };

  private onKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      this.props.onSubmitValue?.(this.state.value);
      this.inputRef.current?.blur();
    }
    this.props.onKeyPress?.(event);
  };
}

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

export function AutoSizingTextInput(props: IInputProps) {
  const { onChangeValue, onFocus, onBlur, ...otherProps } = props;

  const [value, setValue] = useState(otherProps.value ?? '');
  const [focused, setFocused, setBlurred] = useBoolean(false);
  const inputRef = useRef() as React.RefObject<Input>;

  const onChangeValueWrapper = useCallback(
    (value: string) => {
      setValue(value);
      onChangeValue?.(value);
    },
    [onChangeValue],
  );

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

  return (
    <BackAction disabled={!focused} action={blur}>
      <InputFrame focused={focused}>
        <StyledAutoSizingTextInputContainer>
          <StyledAutoSizingTextInputWrapper>
            <Input
              ref={inputRef}
              onChangeValue={onChangeValueWrapper}
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
