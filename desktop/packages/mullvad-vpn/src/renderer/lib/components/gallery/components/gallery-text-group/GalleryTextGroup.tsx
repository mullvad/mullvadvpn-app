import { Flex, FlexProps } from '../../../flex';

export type GalleryTextGroupProps = FlexProps;

export function GalleryTextGroup({ children, ...props }: GalleryTextGroupProps) {
  return (
    <Flex $gap="medium" $flexDirection="column" {...props}>
      {children}
    </Flex>
  );
}
