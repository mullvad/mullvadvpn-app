import { Flex, FlexProps } from '../flex';

type FlexRowProps = Omit<FlexProps, 'flexDirection'>;

export const FlexRow = (props: FlexRowProps) => <Flex flexDirection="row" {...props} />;
