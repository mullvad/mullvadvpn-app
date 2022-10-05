import React, { useCallback, useContext, useMemo, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import log from '../../shared/logging';
import { useMounted } from '../lib/utilityHooks';
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
import ImageView from './ImageView';

interface ILabelProps {
  textOffset?: number;
  children?: React.ReactNode;
}

export function Label(props: ILabelProps) {
  return <StyledLabel textOffset={props.textOffset ?? 0}>{props.children}</StyledLabel>;
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
