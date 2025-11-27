import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { useBoolean } from '../../../../../../../lib/utility-hooks';
import { TroubleshootingModal } from '../../../troubleshooting-modal';
import { FooterText } from '../footer-text';

export function DefaultLaunchFooter() {
  const [dialogOpen, showDialog, hideDialog] = useBoolean();

  return (
    <>
      <FlexColumn gap="medium">
        <FooterText>
          {
            // TRANSLATORS: Message in launch view when the mullvad service cannot be contacted.
            messages.pgettext(
              'launch-view',
              'Unable to contact the Mullvad system service, your connection might be unsecure. Please troubleshoot or send a problem report by clicking the "Learn more" button.',
            )
          }
        </FooterText>
        <Button onClick={showDialog}>
          <Button.Text>{messages.gettext('Learn more')}</Button.Text>
        </Button>
      </FlexColumn>
      <TroubleshootingModal isOpen={dialogOpen} onClose={hideDialog} />
    </>
  );
}
