import React, { useCallback, useContext, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
import { mediumText } from '../common-styles';
import { CellDisabledContext, Container } from './Container';
import StandaloneSwitch from '../Switch';
import ImageView from '../ImageView';

export const Switch = React.forwardRef(function SwitchT(
  props: StandaloneSwitch['props'],
  ref: React.Ref<HTMLDivElement>,
) {
  const disabled = useContext(CellDisabledContext);
  return <StandaloneSwitch forwardedRef={ref} disabled={disabled} {...props} />;
});

export const InputFrame = styled.div({
  flexGrow: 0,
  backgroundColor: 'rgba(255,255,255,0.1)',
  borderRadius: '4px',
  padding: '4px 8px',
});

const inputTextStyles: React.CSSProperties = {
  ...mediumText,
  fontWeight: 600,
  height: '28px',
  textAlign: 'right',
  padding: '0px',
};

const StyledInput = styled.input({}, (props: { valid?: boolean }) => ({
  ...inputTextStyles,
  backgroundColor: 'transparent',
  border: 'none',
  width: '100%',
  height: '100%',
  color: props.valid !== false ? colors.white : colors.red,
  '::placeholder': {
    color: colors.white60,
  },
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
    if (this.props.submitOnBlur) {
      this.props.onSubmitValue?.(this.state.value);
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

export function AutoSizingTextInput({ onChangeValue, ...otherProps }: IInputProps) {
  const [value, setValue] = useState(otherProps.value ?? '');

  const onChangeValueWrapper = useCallback(
    (value: string) => {
      setValue(value);
      onChangeValue?.(value);
    },
    [onChangeValue],
  );

  return (
    <StyledAutoSizingTextInputContainer>
      <StyledAutoSizingTextInputWrapper>
        <Input onChangeValue={onChangeValueWrapper} {...otherProps} />
      </StyledAutoSizingTextInputWrapper>
      <StyledAutoSizingTextInputFiller className={otherProps.className} aria-hidden={true}>
        {value === '' ? otherProps.placeholder : value}
      </StyledAutoSizingTextInputFiller>
    </StyledAutoSizingTextInputContainer>
  );
}

const StyledCellInputRowContainer = styled(Container)({
  backgroundColor: 'white',
  marginBottom: '1px',
});

const StyledSubmitButton = styled.button({
  border: 'none',
  backgroundColor: 'transparent',
  padding: '14px 0',
});

const StyledInputWrapper = styled.div({}, (props: { marginLeft: number }) => ({
  position: 'relative',
  flex: 1,
  width: '171px',
  marginLeft: props.marginLeft + 'px',
  marginRight: '25px',
  lineHeight: '24px',
  minHeight: '24px',
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
  padding: '14px 0',
  maxWidth: '100%',
}));

const StyledTextArea = styled.textarea({}, (props: { invalid?: boolean }) => ({
  position: 'absolute',
  top: 0,
  left: 0,
  width: '100%',
  height: '100%',
  backgroundColor: 'transparent',
  border: 'none',
  flex: 1,
  lineHeight: '24px',
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
  resize: 'none',
  padding: '14px 0',
  color: props.invalid ? colors.red : 'auto',
}));

const StyledInputFiller = styled.div({
  whiteSpace: 'pre-wrap',
  overflowWrap: 'break-word',
  minHeight: '24px',
  color: 'transparent',
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

  const globalKeyListener = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        props.onBlur?.();
      }
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

  useEffect(() => {
    document.addEventListener('keydown', globalKeyListener, true);
    return () => document.removeEventListener('keydown', globalKeyListener, true);
  }, []);

  return (
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
          onFocus={props.onFocus}
          onBlur={props.onBlur}
          placeholder={props.placeholder}
        />
      </StyledInputWrapper>
      <StyledSubmitButton onClick={submit}>
        <ImageView
          source="icon-tick"
          height={22}
          tintColor={colors.green}
          tintHoverColor={colors.green90}
        />
      </StyledSubmitButton>
    </StyledCellInputRowContainer>
  );
}
