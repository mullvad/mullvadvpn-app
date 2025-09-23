import styled from 'styled-components';

import { Text, TextProps } from '../../text';
import { useLinkContext } from '../LinkContext';

export type LinkTextProps = TextProps;

export const StyledLinkText = styled(Text)`
  text-decoration-line: underline;
  text-underline-offset: 2px;
`;

export function LinkText(props: LinkTextProps) {
  const { variant, color } = useLinkContext();
  return <StyledLinkText variant={variant} color={color} {...props}></StyledLinkText>;
}
