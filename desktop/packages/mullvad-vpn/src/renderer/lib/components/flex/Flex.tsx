import React from 'react';
import styled, { css } from 'styled-components';

import { Spacings, spacings } from '../../foundations';
import { TransientProps } from '../../types';
import { Layout, LayoutProps } from '../layout';

type FlexStyleProps = {
  gap?: Spacings;
  flexDirection?: React.CSSProperties['flexDirection'];
  alignItems?: React.CSSProperties['alignItems'];
  justifyContent?: React.CSSProperties['justifyContent'];
  flexGrow?: React.CSSProperties['flexGrow'];
  flexShrink?: React.CSSProperties['flexShrink'];
  flexBasis?: React.CSSProperties['flexBasis'];
  flexWrap?: React.CSSProperties['flexWrap'];
  alignSelf?: React.CSSProperties['alignSelf'];
};

export type FlexProps<T extends React.ElementType = 'div'> = LayoutProps<T> & FlexStyleProps;

const StyledFlex = styled(Layout)<TransientProps<FlexStyleProps>>(({
  $gap: gapProp,
  $flexDirection,
  $alignItems,
  $justifyContent,
  $flexGrow,
  $flexShrink,
  $flexBasis,
  $flexWrap,
  $alignSelf,
}) => {
  const $gap = gapProp ? spacings[gapProp] : undefined;
  return css`
    display: flex;
    gap: ${$gap};
    flex-direction: ${$flexDirection};
    align-items: ${$alignItems};
    justify-content: ${$justifyContent};
    flex-grow: ${$flexGrow};
    flex-shrink: ${$flexShrink};
    flex-basis: ${$flexBasis};
    flex-wrap: ${$flexWrap};
    align-self: ${$alignSelf};
  `;
});

export function Flex<T extends React.ElementType = 'div'>({
  as,
  gap,
  flexDirection,
  alignItems,
  justifyContent,
  flexGrow,
  flexShrink,
  flexBasis,
  flexWrap,
  alignSelf,
  ...props
}: FlexProps<T>) {
  return (
    <StyledFlex
      forwardedAs={as}
      $gap={gap}
      $flexDirection={flexDirection}
      $alignItems={alignItems}
      $justifyContent={justifyContent}
      $flexGrow={flexGrow}
      $flexShrink={flexShrink}
      $flexBasis={flexBasis}
      $flexWrap={flexWrap}
      $alignSelf={alignSelf}
      {...props}
    />
  );
}
