import { AlertTitleProps } from '../../../alert/components';
import { Text } from '../../../text';
import { useDialogContext } from '../../DialogContext';

export type DialogSubtitleProps = AlertTitleProps;

export function DialogTitle({ children, ...props }: DialogSubtitleProps) {
  const { titleId } = useDialogContext();
  return (
    <Text id={titleId} variant="titleLarge" color="white" as="h2" {...props}>
      {children}
    </Text>
  );
}
