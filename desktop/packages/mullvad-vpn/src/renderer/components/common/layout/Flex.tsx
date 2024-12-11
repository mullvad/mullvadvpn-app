import React from 'react';
import styled from 'styled-components';

import { Spacings } from '../../../tokens';
import { Layout, LayoutProps } from './Layout';

export interface FlexProps extends LayoutProps {
  $gap?: Spacings;
  $flex?: React.CSSProperties['flex'];
  $flexDirection?: React.CSSProperties['flexDirection'];
  $alignItems?: React.CSSProperties['alignItems'];
  $justifyContent?: React.CSSProperties['justifyContent'];
  $flexGrow?: React.CSSProperties['flexGrow'];
  $flexShrink?: React.CSSProperties['flexShrink'];
  $flexBasis?: React.CSSProperties['flexBasis'];
  children?: React.ReactNode;
}

export const Flex = styled(Layout)<FlexProps>(
  ({
    $gap,
    $flex,
    $flexDirection,
    $alignItems,
    $justifyContent,
    $flexGrow,
    $flexShrink,
    $flexBasis,
  }) => ({
    display: 'flex',
    gap: $gap,
    flex: $flex,
    flexDirection: $flexDirection,
    alignItems: $alignItems,
    justifyContent: $justifyContent,
    flexGrow: $flexGrow,
    flexShrink: $flexShrink,
    flexBasis: $flexBasis,
  }),
);
