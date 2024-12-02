import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../tokens';

export interface FlexProps {
  $gap?: keyof typeof spacings;
  $flex?: React.CSSProperties['flex'];
  $flexDirection?: React.CSSProperties['flexDirection'];
  $alignItems?: React.CSSProperties['alignItems'];
  $justifyContent?: React.CSSProperties['justifyContent'];
  $flexGrow?: React.CSSProperties['flexGrow'];
  $flexShrink?: React.CSSProperties['flexShrink'];
  children?: React.ReactNode;
}

export const Flex = styled.div<FlexProps>(
  ({ $gap, $flex, $flexDirection, $alignItems, $justifyContent, $flexGrow, $flexShrink }) => ({
    display: 'flex',
    gap: $gap ? spacings[$gap] : undefined,
    flex: $flex,
    flexDirection: $flexDirection,
    alignItems: $alignItems,
    justifyContent: $justifyContent,
    flexGrow: $flexGrow,
    flexShrink: $flexShrink,
  }),
);
