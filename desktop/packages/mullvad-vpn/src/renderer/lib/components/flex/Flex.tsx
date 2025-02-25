import React from 'react';
import styled from 'styled-components';

import { Spacings } from '../../foundations';
import { Layout, LayoutProps } from '../layout';

export interface FlexProps extends LayoutProps {
  $gap?: Spacings;
  $flex?: React.CSSProperties['flex'];
  $flexDirection?: React.CSSProperties['flexDirection'];
  $alignItems?: React.CSSProperties['alignItems'];
  $justifyContent?: React.CSSProperties['justifyContent'];
  $flexGrow?: React.CSSProperties['flexGrow'];
  $flexShrink?: React.CSSProperties['flexShrink'];
  $flexBasis?: React.CSSProperties['flexBasis'];
  $flexWrap?: React.CSSProperties['flexWrap'];
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
    $flexWrap,
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
    flexWrap: $flexWrap,
  }),
);
