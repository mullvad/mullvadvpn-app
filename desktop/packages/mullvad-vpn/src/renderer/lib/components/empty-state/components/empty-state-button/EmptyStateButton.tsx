import { Button, ButtonProps } from '../../../button';
import { useEmptyStateContext } from '../../EmptyStateContext';

export type EmpptyStateButtonProps = ButtonProps;

function EmptyStateButton({ children, ...props }: EmpptyStateButtonProps) {
  const { variant } = useEmptyStateContext();
  const disabled = variant === 'loading';
  return (
    <Button disabled={disabled} {...props}>
      {children}
    </Button>
  );
}

const EmptyStateButtonNamespace = Object.assign(EmptyStateButton, {
  Text: Button.Text,
});

export { EmptyStateButtonNamespace as EmptyStateButton };
