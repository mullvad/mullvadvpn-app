import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';

export type PauseButtonProps = {
  children?: React.ReactNode;
  disabled?: boolean;
};

export function PauseButton({ children, disabled }: PauseButtonProps) {
  const { appUpgradeAbort } = useAppContext();

  return (
    <Button disabled={disabled} onClick={appUpgradeAbort}>
      <Button.Text>{children}</Button.Text>
    </Button>
  );
}
