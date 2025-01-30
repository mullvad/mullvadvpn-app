import React, { useCallback, useContext, useMemo, useState } from 'react';
import styled from 'styled-components';

import log from '../../shared/logging';
import { Colors } from '../lib/foundations';
import { useMounted } from '../lib/utility-hooks';
import {
  StyledButtonContent,
  StyledHiddenSide,
  StyledLabel,
  StyledLeft,
  StyledRight,
  StyledVisibleSide,
  transparentButton,
} from './AppButtonStyles';
import { measurements } from './common-styles';

interface ILabelProps {
  textOffset?: number;
  children?: React.ReactNode;
}

export function Label(props: ILabelProps) {
  return <StyledLabel $textOffset={props.textOffset ?? 0}>{props.children}</StyledLabel>;
}

export interface IProps extends React.HTMLAttributes<HTMLButtonElement> {
  children?: React.ReactNode;
  className?: string;
  disabled?: boolean;
  onClick?: () => void;
  textOffset?: number;
}

type ChildrenGroups = { left: React.ReactNode[]; label: React.ReactNode; right: React.ReactNode[] };

const BaseButton = React.memo(function BaseButtonT(props: IProps) {
  const { children, textOffset, ...otherProps } = props;

  const groupedChildren = useMemo(() => {
    return React.Children.toArray(children).reduce(
      (groups: ChildrenGroups, child) => {
        if (groups.label === undefined && typeof child === 'string') {
          return { ...groups, label: <Label textOffset={textOffset}>{child}</Label> };
        } else if (React.isValidElement(child) && child.type === Label) {
          return {
            ...groups,
            label: React.cloneElement(child as React.ReactElement<ILabelProps>, { textOffset }),
          };
        } else if (groups.label === undefined) {
          return { ...groups, left: [...groups.left, child] };
        } else {
          return { ...groups, right: [...groups.right, child] };
        }
      },
      { left: [], label: undefined, right: [] },
    );
  }, [children, textOffset]);

  return (
    <StyledSimpleButton {...otherProps}>
      <StyledButtonContent>
        <StyledLeft>
          <StyledVisibleSide>{groupedChildren.left}</StyledVisibleSide>
          <StyledHiddenSide>{groupedChildren.right}</StyledHiddenSide>
        </StyledLeft>

        {groupedChildren.label ?? <Label />}

        <StyledRight>
          <StyledVisibleSide>{groupedChildren.right}</StyledVisibleSide>
          <StyledHiddenSide>{groupedChildren.left}</StyledHiddenSide>
        </StyledRight>
      </StyledButtonContent>
    </StyledSimpleButton>
  );
});

function SimpleButtonT(props: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  const blockingContext = useContext(BlockingContext);

  return (
    <button
      {...props}
      disabled={props.disabled || blockingContext.disabled}
      onClick={blockingContext.onClick ?? props.onClick}>
      {props.children}
    </button>
  );
}

export const SimpleButton = React.memo(SimpleButtonT);

const StyledSimpleButton = styled(SimpleButton)({
  display: 'flex',
  cursor: 'default',
  borderRadius: 4,
  border: 'none',
  padding: 0,
  '&&:disabled': {
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
  const { onClick: propsOnClick } = props;

  const isMounted = useMounted();
  const [isBlocked, setIsBlocked] = useState(false);

  const onClick = useCallback(async () => {
    setIsBlocked(true);
    try {
      await propsOnClick();
    } catch (error) {
      log.error(`onClick() failed - ${error}`);
    }

    if (isMounted()) {
      setIsBlocked(false);
    }
  }, [isMounted, propsOnClick]);

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
  backgroundColor: Colors.red,
  '&&:not(:disabled):hover': {
    backgroundColor: Colors.red95,
  },
});

export const GreenButton = styled(BaseButton)({
  backgroundColor: Colors.green,
  '&&:not(:disabled):hover': {
    backgroundColor: Colors.green90,
  },
});

export const BlueButton = styled(BaseButton)({
  backgroundColor: Colors.blue80,
  '&&:not(:disabled):hover': {
    backgroundColor: Colors.blue60,
  },
});

export const TransparentButton = styled(BaseButton)(transparentButton, {
  backgroundColor: Colors.white20,
  '&&:not(:disabled):hover': {
    backgroundColor: Colors.white40,
  },
});

export const RedTransparentButton = styled(BaseButton)(transparentButton, {
  backgroundColor: Colors.red60,
  '&&:not(:disabled):hover': {
    backgroundColor: Colors.red80,
  },
});

const StyledButtonWrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  '&&:not(:last-child)': {
    marginBottom: measurements.buttonVerticalMargin,
  },
});

interface IButtonGroupProps {
  children: React.ReactNode | React.ReactNode[];
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
