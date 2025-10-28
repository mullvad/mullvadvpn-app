import { Text, TextProps } from '../../../text';

export type GalleryTextProps = TextProps;

export function GalleryText({ children, ...props }: GalleryTextProps) {
  return (
    <Text variant="labelTiny" color="whiteAlpha60" {...props}>
      {children}
    </Text>
  );
}
