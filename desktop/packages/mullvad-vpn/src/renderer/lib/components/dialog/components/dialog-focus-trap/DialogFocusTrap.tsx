import { FocusTrap } from 'focus-trap-react';

import { useDialogContext } from '../../DialogContext';

const FALLBACK_FOCUS_CLASSNAME = 'dialog-fallback-focus';

export function DialogFocusTrap({ children }: React.PropsWithChildren) {
  const { open } = useDialogContext();

  return (
    <FocusTrap
      active={open}
      focusTrapOptions={{
        fallbackFocus: `.${FALLBACK_FOCUS_CLASSNAME}`,
      }}>
      <div>
        <div className={FALLBACK_FOCUS_CLASSNAME} tabIndex={-1} />
        {children}
      </div>
    </FocusTrap>
  );
}
