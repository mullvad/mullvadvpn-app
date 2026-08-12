import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Flex, FlexProps } from '../flex';
import { Image, StyledImage } from '../image';
import { GalleryText, GalleryTextGroup, StyledGalleryTextGroup } from './components';

export type GalleryProps = FlexProps;

export const StyledGallery = styled(Flex)`
  height: 100%;
  min-height: 0;

  &:has(${StyledImage} + ${StyledGalleryTextGroup}) {
    ${StyledImage} {
      margin-bottom: ${spacings.small};
    }
  }
`;

function Gallery({ children, ...props }: GalleryProps) {
  return (
    <StyledGallery flexDirection="column" {...props}>
      {children}
    </StyledGallery>
  );
}

const GalleryNamespace = Object.assign(Gallery, {
  Image: Image,
  Text: GalleryText,
  TextGroup: GalleryTextGroup,
});

export { GalleryNamespace as Gallery };
