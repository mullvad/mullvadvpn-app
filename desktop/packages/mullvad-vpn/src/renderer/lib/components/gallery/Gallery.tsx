import { Flex, FlexProps } from '../flex';
import { Image } from '../image';
import { GalleryText, GalleryTextGroup } from './components';

export type GalleryProps = FlexProps;

function Gallery({ children, ...props }: GalleryProps) {
  return (
    <Flex flexDirection="column" gap="small" {...props}>
      {children}
    </Flex>
  );
}

const GalleryNamespace = Object.assign(Gallery, {
  Image: Image,
  Text: GalleryText,
  TextGroup: GalleryTextGroup,
});

export { GalleryNamespace as Gallery };
