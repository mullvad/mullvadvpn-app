import styled from 'styled-components';

import { Flex, FlexProps } from '../../../flex';

export type GalleryTextGroupProps = FlexProps;

export const StyledGalleryTextGroup = styled(Flex)``;

export function GalleryTextGroup({ children, ...props }: GalleryTextGroupProps) {
  return (
    <StyledGalleryTextGroup gap="medium" flexDirection="column" {...props}>
      {children}
    </StyledGalleryTextGroup>
  );
}
