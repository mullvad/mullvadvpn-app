import { Flex, FlexProps } from '../flex';

type RowProps = Omit<FlexProps, '$flexDirection'>;

export const Row = (props: RowProps) => <Flex $flexDirection="row" {...props} />;
