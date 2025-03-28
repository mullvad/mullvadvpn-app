import { Flex, FlexProps } from '../flex';

type ColumnProps = Omit<FlexProps, '$flexDirection'>;

export const Column = (props: ColumnProps) => <Flex $flexDirection="column" {...props} />;
