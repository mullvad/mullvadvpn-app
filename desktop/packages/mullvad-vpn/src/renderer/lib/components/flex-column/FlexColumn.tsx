import { Flex, FlexProps } from '../flex';

type FlexColumnProps = Omit<FlexProps, '$flexDirection'>;

export const FlexColumn = (props: FlexColumnProps) => <Flex $flexDirection="column" {...props} />;
