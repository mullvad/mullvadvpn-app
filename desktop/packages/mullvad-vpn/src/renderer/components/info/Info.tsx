import type { DialogProps } from '../../lib/components/dialog';
import { InfoButton, InfoDialog } from './components';
import { InfoProvider } from './InfoContext';

export type InfoProps = React.PropsWithChildren & {
  open?: DialogProps['open'];
  onOpenChange?: DialogProps['onOpenChange'];
};

function Info({ open, onOpenChange, children }: InfoProps) {
  return (
    <InfoProvider open={open} onOpenChange={onOpenChange}>
      {children}
    </InfoProvider>
  );
}

const InfoNamespace = Object.assign(Info, {
  Dialog: InfoDialog,
  Button: InfoButton,
});

export { InfoNamespace as Info };
