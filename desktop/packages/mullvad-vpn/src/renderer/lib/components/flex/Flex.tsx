import React from 'react';
import styled from 'styled-components';

import { Spacings, spacings } from '../../foundations';
import { TransientProps } from '../../types';
import { Layout, LayoutProps } from '../layout';

export type FlexProps = LayoutProps & {
  gap?: Spacings;
  flex?: React.CSSProperties['flex'];
  flexDirection?: React.CSSProperties['flexDirection'];
  alignItems?: React.CSSProperties['alignItems'];
  justifyContent?: React.CSSProperties['justifyContent'];
  flexGrow?: React.CSSProperties['flexGrow'];
  flexShrink?: React.CSSProperties['flexShrink'];
  flexBasis?: React.CSSProperties['flexBasis'];
  flexWrap?: React.CSSProperties['flexWrap'];
  alignSelf?: React.CSSProperties['alignSelf'];
};

const StyledFlex = styled(Layout)<TransientProps<FlexProps>>(({
  $gap: gapProp,
  $flex,
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
  return {
    display: 'flex',
    gap: $gap,
    flex: $flex,
    flexDirection: $flexDirection,
    alignItems: $alignItems,
    justifyContent: $justifyContent,
    flexGrow: $flexGrow,
    flexShrink: $flexShrink,
    flexBasis: $flexBasis,
    flexWrap: $flexWrap,
    alignSelf: $alignSelf,
  };
});

export function Flex({
  gap,
  flex,
  flexDirection,
  alignItems,
  justifyContent,
  flexGrow,
  flexShrink,
  flexBasis,
  flexWrap,
  alignSelf,
  ...props
}: FlexProps) {
  return (
    <StyledFlex
      $gap={gap}
      $flex={flex}
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
