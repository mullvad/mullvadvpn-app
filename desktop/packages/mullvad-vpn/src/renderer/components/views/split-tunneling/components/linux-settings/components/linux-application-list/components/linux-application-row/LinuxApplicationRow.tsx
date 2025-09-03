import { Icon, Label, LaunchButton, WarningDialog, WarningIcon } from './components';
import { useShowWarningIcon } from './hooks';
import { LinuxApplicationRowContextProvider } from './LinuxApplicationRowContext';
import { type LinuxApplicationRowProps } from './types';

function LinuxApplicationRowInner() {
  const showWarningIcon = useShowWarningIcon();

  return (
    <>
      <LaunchButton>
        <Icon />
        <Label />
      </LaunchButton>
      {showWarningIcon && <WarningIcon />}
      <WarningDialog />
    </>
  );
}

export function LinuxApplicationRow(props: LinuxApplicationRowProps) {
  return (
    <LinuxApplicationRowContextProvider {...props}>
      <LinuxApplicationRowInner />
    </LinuxApplicationRowContextProvider>
  );
}
