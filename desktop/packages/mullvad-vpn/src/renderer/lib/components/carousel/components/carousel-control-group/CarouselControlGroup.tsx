import { Flex } from '../../../flex';

export type CarouselControlGroupProps = React.ComponentPropsWithRef<'div'>;

export function CarouselControlGroup({ children, ...props }: CarouselControlGroupProps) {
  return (
    <Flex gap="small" {...props}>
      {children}
    </Flex>
  );
}
