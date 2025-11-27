import { Flex, FlexProps } from '../flex';

export type FlexColumnProps = Omit<FlexProps, 'flexDirection'>;

export const FlexColumn = (props: FlexColumnProps) => <Flex flexDirection="column" {...props} />;
