import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';

type LinkIconProps = IconProps;

export const StyledIcon = styled(Icon)`
  vertical-align: text-bottom;
  display: inline-block;
  flex-shrink: 0;
`;

export function LinkIcon({ ...props }: LinkIconProps) {
  return <StyledIcon size="small" color="chalk" {...props} />;
}
