import { Button, type ButtonProps } from '../button';
import { ProgressButtonStatusIndicator } from './components';
import { ProgressButtonProvider } from './ProgressButtonContext';

export type ProgressButtonStatus = 'idle' | 'loading' | 'success' | 'error';

export type ProgressButtonProps = ButtonProps & {
  status: ProgressButtonStatus;
};

function ProgressButton({ status, children, ...props }: ProgressButtonProps) {
  const disabled = status !== 'idle';
  return (
    <ProgressButtonProvider status={status}>
      <Button disabled={disabled} aria-live="polite" {...props}>
        {children}
      </Button>
    </ProgressButtonProvider>
  );
}

const ProgressButtonNamespace = Object.assign(ProgressButton, {
  Text: Button.Text,
  StatusIndicator: ProgressButtonStatusIndicator,
});

export { ProgressButtonNamespace as ProgressButton };
