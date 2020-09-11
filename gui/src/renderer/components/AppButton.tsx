import log from 'electron-log';
import React, { useContext } from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import {
  StyledButton,
  StyledButtonContent,
  StyledLabel,
  StyledLabelContainer,
} from './AppButtonStyles';
import ImageView from './ImageView';

const ButtonContext = React.createContext({
  textAdjustment: 0,
  textRef: React.createRef<HTMLDivElement>(),
});

interface ILabelProps {
  children?: React.ReactText;
}

export function Label(props: ILabelProps) {
  const { textAdjustment, textRef } = useContext(ButtonContext);
  return (
    <StyledLabelContainer ref={textRef} textAdjustment={textAdjustment}>
      <StyledLabel>{props.children}</StyledLabel>
    </StyledLabelContainer>
  );
}

interface IIconProps {
  source: string;
  width?: number;
  height?: number;
}

export function Icon(props: IIconProps) {
  return <ImageView {...props} tintColor={colors.white} />;
}

export interface IProps extends React.HTMLAttributes<HTMLButtonElement> {
  children?: React.ReactNode;
  className?: string;
  disabled?: boolean;
  onClick?: () => void;
  textOffset?: number;
}

interface IState {
  textAdjustment: number;
}

class BaseButton extends React.Component<IProps, IState> {
  public state: IState = {
    textAdjustment: 0,
  };

  private buttonRef = React.createRef<HTMLButtonElement>();
  private textRef = React.createRef<HTMLDivElement>();

  public componentDidMount() {
    this.updateTextAdjustment();
  }

  public componentDidUpdate() {
    this.updateTextAdjustment();
  }

  public render() {
    const { children, ...otherProps } = this.props;

    return (
      <ButtonContext.Provider
        value={{
          textAdjustment: this.state.textAdjustment,
          textRef: this.textRef,
        }}>
        <StyledButton ref={this.buttonRef} {...otherProps}>
          <StyledButtonContent>
            {React.Children.map(children, (child) =>
              typeof child === 'string' ? <Label>{child as string}</Label> : child,
            )}
          </StyledButtonContent>
        </StyledButton>
      </ButtonContext.Provider>
    );
  }

  private updateTextAdjustment() {
    const textOffset = this.props.textOffset ?? 0;

    const buttonRect = this.buttonRef.current?.getBoundingClientRect();
    const textRect = this.textRef.current?.getBoundingClientRect();

    if (buttonRect && textRect) {
      const leftDiff = textRect.left - buttonRect.left;

      // calculate the remaining space at the right hand side
      const trailingSpace = buttonRect.width - (leftDiff + textRect.width);

      // calculate text adjustment
      const textAdjustment = leftDiff - trailingSpace - textOffset;

      // re-render the view with the new text adjustment if it changed
      if (this.state.textAdjustment !== textAdjustment) {
        this.setState({ textAdjustment });
      }
    }
  }
}

interface IBlockingState {
  isBlocked: boolean;
}

interface IBlockingProps {
  children?: React.ReactNode;
  onClick: () => Promise<void>;
  disabled?: boolean;
}

export class BlockingButton extends React.Component<IBlockingProps, IBlockingState> {
  public state = {
    isBlocked: false,
  };

  public render() {
    return React.Children.map(this.props.children, (child) => {
      if (React.isValidElement(child)) {
        return React.cloneElement(child as React.ReactElement, {
          ...child.props,
          disabled: this.state.isBlocked || this.props.disabled,
          onClick: this.onClick,
        });
      } else {
        return child;
      }
    });
  }

  private onClick = () => {
    this.setState({ isBlocked: true }, async () => {
      try {
        await this.props.onClick();
      } catch (error) {
        log.error(`onClick() failed - ${error}`);
      }
      this.setState({ isBlocked: false });
    });
  };
}

export const RedButton = styled(BaseButton)({
  backgroundColor: colors.red,
  ':not(:disabled):hover': {
    backgroundColor: colors.red95,
  },
});

export const GreenButton = styled(BaseButton)({
  backgroundColor: colors.green,
  ':not(:disabled):hover': {
    backgroundColor: colors.green90,
  },
});

export const BlueButton = styled(BaseButton)({
  backgroundColor: colors.blue80,
  ':not(:disabled):hover': {
    backgroundColor: colors.blue60,
  },
});

export const TransparentButton = styled(BaseButton)({
  backgroundColor: colors.white20,
  ':not(:disabled):hover': {
    backgroundColor: colors.white40,
  },
});

export const RedTransparentButton = styled(BaseButton)({
  backgroundColor: colors.red60,
  ':not(:disabled):hover': {
    backgroundColor: colors.red80,
  },
});
