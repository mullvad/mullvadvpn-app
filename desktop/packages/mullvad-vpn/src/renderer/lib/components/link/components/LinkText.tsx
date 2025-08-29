import styled from 'styled-components';

import { Text, TextProps } from '../../typography';
import { useLinkContext } from '../LinkContext';

export type LinkTextProps = TextProps;

export const StyledLinkText = styled(Text)``;

export function LinkText(props: LinkTextProps) {
  const { variant, color } = useLinkContext();
  return <StyledLinkText variant={variant} color={color} {...props}></StyledLinkText>;
}
