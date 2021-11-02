import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import log from '../../shared/logging';
import { useMounted } from '../lib/utilityHooks';
import {
  StyledButtonContent,
  StyledLabel,
  StyledLabelContainer,
  transparentButton,
} from './AppButtonStyles';
import ImageView from './ImageView';

interface IButtonContext {
  textAdjustment: number;
  textRef?: React.Ref<HTMLDivElement>;
}

const ButtonContext = React.createContext<IButtonContext>({
  textAdjustment: 0,
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

const BaseButton = React.memo(function BaseButtonT(props: IProps) {
  const { children, textOffset, ...otherProps } = props;

  const [textAdjustment, setTextAdjustment] = useState(0);
  const buttonRef = useRef() as React.RefObject<HTMLButtonElement>;
  const textRef = useRef() as React.RefObject<HTMLDivElement>;

  const contextValue = useMemo(() => ({ textAdjustment, textRef }), [textAdjustment, textRef]);

  useEffect(() => {
    const buttonRect = buttonRef.current?.getBoundingClientRect();
    const textRect = textRef.current?.getBoundingClientRect();

    if (buttonRect && textRect) {
      const leftDiff = textRect.left - buttonRect.left;

      // calculate the remaining space at the right hand side
      const trailingSpace = buttonRect.width - (leftDiff + textRect.width);

      // calculate text adjustment
      const textAdjustment = leftDiff - trailingSpace - (textOffset ?? 0);

      // re-render the view with the new text adjustment if it changed
      setTextAdjustment(textAdjustment);
    }
  });

  return (
    <ButtonContext.Provider value={contextValue}>
      <StyledSimpleButton ref={buttonRef} {...otherProps}>
        <StyledButtonContent>
          {React.Children.map(children, (child) =>
            typeof child === 'string' ? <Label>{child as string}</Label> : child,
          )}
        </StyledButtonContent>
      </StyledSimpleButton>
    </ButtonContext.Provider>
  );
});

function SimpleButtonT(
  props: React.ButtonHTMLAttributes<HTMLButtonElement>,
  ref: React.Ref<HTMLButtonElement>,
) {
  const blockingContext = useContext(BlockingContext);

  return (
    <button
      ref={ref}
      {...props}
      disabled={props.disabled || blockingContext.disabled}
      onClick={blockingContext.onClick ?? props.onClick}>
      {props.children}
    </button>
  );
}

export const SimpleButton = React.memo(React.forwardRef(SimpleButtonT));

const StyledSimpleButton = styled(SimpleButton)({
  display: 'flex',
  cursor: 'default',
  borderRadius: 4,
  border: 'none',
  padding: 0,
  ':disabled': {
    opacity: 0.5,
  },
});

interface IBlockingContext {
  disabled?: boolean;
  onClick?: () => Promise<void>;
}

const BlockingContext = React.createContext<IBlockingContext>({});

interface IBlockingProps {
  children?: React.ReactNode;
  onClick: () => Promise<void>;
  disabled?: boolean;
}

export function BlockingButton(props: IBlockingProps) {
  const isMounted = useMounted();
  const [isBlocked, setIsBlocked] = useState(false);

  const onClick = useCallback(async () => {
    setIsBlocked(true);
    try {
      await props.onClick();
    } catch (error) {
      log.error(`onClick() failed - ${error}`);
    }

    if (isMounted()) {
      setIsBlocked(false);
    }
  }, [props.onClick]);

  const contextValue = useMemo(
    () => ({
      disabled: isBlocked || props.disabled,
      onClick,
    }),
    [isBlocked, props.disabled, onClick],
  );

  return <BlockingContext.Provider value={contextValue}>{props.children}</BlockingContext.Provider>;
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

export const TransparentButton = styled(BaseButton)(transparentButton, {
  backgroundColor: colors.white20,
  ':not(:disabled):hover': {
    backgroundColor: colors.white40,
  },
});

export const RedTransparentButton = styled(BaseButton)(transparentButton, {
  backgroundColor: colors.red60,
  ':not(:disabled):hover': {
    backgroundColor: colors.red80,
  },
});

const StyledButtonWrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  ':not(:last-child)': {
    marginBottom: '18px',
  },
});

interface IButtonGroupProps {
  children: React.ReactElement[];
}

export function ButtonGroup(props: IButtonGroupProps) {
  return (
    <>
      {React.Children.map(props.children, (button, index) => (
        <StyledButtonWrapper key={index}>{button}</StyledButtonWrapper>
      ))}
    </>
  );
}
