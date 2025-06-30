import { messages } from '../../../../../../../shared/gettext';

export const useTroubleshootingSteps = () => {
  let restartBackgroundProcessStep = <></>;
  if (window.env.platform === 'win32') {
    restartBackgroundProcessStep = (
      <li>
        {
          // TRANSLATORS: List item in troubleshooting modal instructing user how to restart the background process.
          messages.pgettext(
            'launch-view',
            'Restarting the Mullvad background process by clicking "Back", then "Try again"',
          )
        }
      </li>
    );
  } else {
    restartBackgroundProcessStep = (
      <li>
        {
          // TRANSLATORS: List item in troubleshooting modal advising user to restart background process.
          messages.pgettext('launch-view', 'Restarting the Mullvad background process')
        }
      </li>
    );
  }
  return (
    <>
      {restartBackgroundProcessStep}
      <li>
        {
          // TRANSLATORS: List item in troubleshooting modal advising user to restart their computer.
          messages.pgettext('launch-view', 'Restarting your computer')
        }
      </li>
      <li>
        {
          // TRANSLATORS: List item in troubleshooting modal advising user to reinstall the app.
          messages.pgettext('launch-view', 'Reinstalling the app')
        }
      </li>
      <li>
        {
          // TRANSLATORS: List item in troubleshooting modal advising user disable third party antivirus.
          messages.pgettext('launch-view', 'Disabling third party antivirus software')
        }
      </li>
    </>
  );
};
