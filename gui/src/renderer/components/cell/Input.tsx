import React, { useCallback, useContext, useState } from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
import { mediumText } from '../common-styles';
import { CellDisabledContext } from './Container';
import StandaloneSwitch from '../Switch';

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
