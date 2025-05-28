import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';

export type UpgradeButtonProps = {
  children?: React.ReactNode;
  disabled?: boolean;
};

export function UpgradeButton({ children, disabled }: UpgradeButtonProps) {
  const { appUpgrade } = useAppContext();

  return (
    <Button disabled={disabled} onClick={appUpgrade}>
      <Button.Text>{children}</Button.Text>
    </Button>
  );
}
