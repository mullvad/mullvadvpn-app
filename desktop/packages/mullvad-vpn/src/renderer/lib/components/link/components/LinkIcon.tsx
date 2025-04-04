import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';

type LinkIconProps = IconProps;

export const StyledIcon = styled(Icon)`
  vertical-align: middle;
  display: inline-flex;
`;

export function LinkIcon({ ...props }: LinkIconProps) {
  return <StyledIcon size="small" {...props} />;
}
