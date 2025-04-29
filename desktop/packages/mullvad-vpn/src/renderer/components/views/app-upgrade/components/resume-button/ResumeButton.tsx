import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';

export type ResumeButtonProps = {
  children?: React.ReactNode;
  disabled?: boolean;
};

export function ResumeButton({ children, disabled }: ResumeButtonProps) {
  const { appUpgrade } = useAppContext();

  return (
    <Button disabled={disabled} onClick={appUpgrade}>
      <Button.Text>{children}</Button.Text>
    </Button>
  );
}
