import log from 'electron-log';
import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { useMounted } from '../lib/utilityHooks';
import {
  StyledButton,
  StyledButtonContent,
  StyledLabel,
  StyledLabelContainer,
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
  const { children, disabled, onClick, ...otherProps } = props;

  const blockingContext = useContext(BlockingContext);
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
      const textOffset = props.textOffset ?? 0;
      const textAdjustment = leftDiff - trailingSpace - textOffset;

      // re-render the view with the new text adjustment if it changed
      setTextAdjustment(textAdjustment);
    }
  });

  return (
    <ButtonContext.Provider value={contextValue}>
      <StyledButton
        ref={buttonRef}
        disabled={blockingContext.disabled || disabled}
        onClick={blockingContext.onClick ?? onClick}
        {...otherProps}>
        <StyledButtonContent>
          {React.Children.map(children, (child) =>
            typeof child === 'string' ? <Label>{child as string}</Label> : child,
          )}
        </StyledButtonContent>
      </StyledButton>
    </ButtonContext.Provider>
  );
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
