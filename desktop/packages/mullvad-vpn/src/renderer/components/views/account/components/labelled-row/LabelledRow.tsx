import { Text } from '../../../../../lib/components';
import { FlexColumn, FlexColumnProps } from '../../../../../lib/components/flex-column';

type LabelledRowProps = FlexColumnProps & {
  label?: string;
};

export function LabelledRow({ label, children, ...props }: LabelledRowProps) {
  return (
    <FlexColumn gap="tiny" {...props}>
      <Text variant="labelTiny" color="whiteAlpha60">
        {label}
      </Text>
      {children}
    </FlexColumn>
  );
}
