import { Button, type ButtonProps } from '../button';

export type ProgressButtonStatus = 'idle' | 'loading' | 'success' | 'error';

export type ProgressButtonProps = ButtonProps & {
  status: ProgressButtonStatus;
};

const leadingElements: Record<ProgressButtonStatus, React.ReactNode> = {
  idle: null,
  loading: <Button.Spinner />,
  success: <Button.Icon icon="checkmark" disabled={false} />,
  error: <Button.Icon icon="cross" disabled={false} />,
};

export function ProgressButton({ status, children, ...props }: ProgressButtonProps) {
  const leadingElement = leadingElements[status];
  const disabled = status !== 'idle';
  return (
    <Button disabled={disabled} aria-live="polite" {...props}>
      {leadingElement}
      <Button.Text>{children}</Button.Text>
    </Button>
  );
}
